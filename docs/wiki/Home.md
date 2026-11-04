# Ultimate MUGEN Wiki

**English** · [Português (BR)](pt-BR/Home.md)

A guide for players and creators who want to customize the engine: install, configure, create fighters, stages and scripts.

| Page | Audience | Covers |
|---|---|---|
| [Installation](Installation.md) | Players | Download, build, run on desktop, browser and phone |
| [Configuration and controls](Configuration.md) | Players | `.cfg` file, command-line flags, environment variables, controls |
| [Adding M.U.G.E.N content](MUGEN-Content.md) | Players | Install `.def`/`.zip` characters and stages, and validate them |
| [GitHub marketplace](Marketplace.md) | Players | Install characters, stages and music from any GitHub repository |
| [Creating a UMF character](UMF-Character.md) | Creators | A full fighter in a single JSON |
| [Skeleton art (rig)](Rig.md) | Creators | Style, poses, joints and palettes |
| [CNS scripts](CNS-Scripts.md) | Creators | States, controllers, triggers and commands |
| [AssertSpecial flags](AssertSpecial.md) | Creators | All 77 Ikemen flags and what each one does |
| [Creating stages](Stages.md) | Creators | M.U.G.E.N `.def` stages and native stages |
| [Customizing the engine](Engine.md) | Developers | Architecture, new controller, new trigger, tests |
| [Performance](Performance.md) | Developers | Parallel loading, frame pacing, benchmarks |
| [Troubleshooting](Troubleshooting.md) | Everyone | Common errors and diagnostic tools |

## Accepted formats

| Format | Use | Files |
|---|---|---|
| UMF | Native declarative fighter | `assets/chars/<id>.json` |
| M.U.G.E.N | Classic fighters and stages | `.def`, `.sff` v1/v2, `.air`, `.cmd`, `.cns`, `.snd`, `.act` |

UMF and M.U.G.E.N fighters run on the same state machine and can fight each other.

## Roadmap

Compatibility plans and architecture decisions are in [`docs/ROADMAP.md`](../ROADMAP.md). Items marked **planned** in this wiki do not exist in the current version yet.

## Acknowledgments

Thanks to the [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) team and to Elecbyte, creator of M.U.G.E.N, for the inspiration.
