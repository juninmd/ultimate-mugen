# Ultimate MUGEN Wiki

**English** · [Português (BR)](pt-BR/Home.md)

A guide for players and creators who want to customize the engine: install, configure, create fighters, stages and scripts.

| Page | Audience | Covers |
|---|---|---|
| [Installation](Installation.md) | Players | Download, build, run on desktop, browser and phone |
| [Configuration and controls](Configuration.md) | Players | `.cfg` file, command-line flags, environment variables, controls |
| [Adding M.U.G.E.N content](MUGEN-Content.md) | Players | Install `.def`/`.zip` characters and stages, and validate them |
| [GitHub marketplace](Marketplace.md) | Players | Install characters, stages and music from any GitHub repository |
| [Game modes](Game-Modes.md) | Players | Arcade, teams (Sequence, Parallel, Tag) and their rules |
| [Compatibility decisions](Compatibility-Decisions.md) | Creators | Features the engine rejects by design and why |
| [Creating a UMF character](UMF-Character.md) | Creators | A full fighter in a single JSON |
| [Skeleton art (rig)](Rig.md) | Creators | Style, poses, joints and palettes |
| [CNS scripts](CNS-Scripts.md) | Creators | States, controllers, triggers and commands |
| [AssertSpecial flags](AssertSpecial.md) | Creators | All 77 Ikemen flags and what each one does |
| [Creating stages](Stages.md) | Creators | M.U.G.E.N `.def` stages and native stages |
| [Customizing the engine](Engine.md) | Developers | Architecture, new controller, new trigger, tests |
| [Performance](Performance.md) | Developers | Parallel loading, frame pacing, benchmarks |
| [Research: fighting games](Research-Fighting-Games.md) | Developers | Survey of other fighters and prioritized improvement list |
| [Stress and resilience](Stress-and-Resilience.md) | Developers | Fuzzing, spawner caps, hot path numbers |
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
