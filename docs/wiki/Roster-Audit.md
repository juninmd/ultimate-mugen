# Roster audit

`ume-tools roster-audit` loads every character and stage of an installed M.U.G.E.N collection, plays real CPU fights
between them and writes a machine-readable report of everything that is broken: loading, visuals (missing or invisible
sprites), sound (calls with no sample), mechanics (stuck states, fighters below the floor, endless attacks, watchdog
rescues), the browser-specific import checks and the memory each fighter needs.

It is the reproducible replacement for "open the game and try a few characters by hand".

## Running it

```bash
cargo build --release -p ume-devtools
target/release/ume-tools roster-audit                     # mugen/chars + mugen/stages -> docs/evidence/homologation
target/release/ume-tools roster-audit --only "goku"        # one family of characters (stages still all run)
target/release/ume-tools roster-audit --limit 20 --no-fights   # a quick static pass
target/release/ume-tools roster-audit --opponent "vegeta"  # force every fight against one opponent
target/release/ume-tools roster-audit --fail-over critical # exit 2 on any critical finding (CI gate)
```

| Flag | Meaning |
|---|---|
| `--chars <dir>` / `--stages <dir>` | roots to scan (default `mugen/chars`, `mugen/stages`) |
| `--out <dir>` | where `report.json`, `report.md` and `issues.csv` are written |
| `--limit N` / `--stage-limit N` | audit only the first N characters / stages |
| `--fights N` | fights per character (each against a different opponent and stage, default 2) |
| `--stage-fights N` | opponent rotation step when a stage is the subject |
| `--only`, `--only-stage`, `--opponent` | substring filters for characters, stages and the forced opponent |
| `--no-fights`, `--quiet` | static pass only, and no per-item progress on stderr |
| `--fail-over <severity>` | exit code 2 when a finding at that severity or worse exists |

Runs are parallel (one load per character per worker, never the whole collection in memory) and every fight is wrapped
in `catch_unwind`, so one broken pack cannot abort the audit.

## What a fight checks

Each fight runs 30 seconds of CPU versus CPU on a real stage with the stage background running:

| Code | Severity | Meaning |
|---|---|---|
| `fight-panic` | critical | the match panicked while ticking (the message is kept in the report) |
| `fight-non-finite` | critical | position, velocity, life or power became NaN/inf |
| `fight-life-range`, `fight-power-range` | major | life or power left its own bounds |
| `fight-below-floor-p1/p2` | major | the fighter stayed more than 12 px under the floor for more than 20 ticks |
| `fight-stuck-p1/p2` | major | 420 ticks without control outside the lying states |
| `fight-endless-attack-p1/p2` | major | 240 ticks stuck inside one attack state |
| `fight-no-contact` | major | the pair never landed a hit in 30 seconds |
| `fight-off-camera-p1/p2` | major | the fighter sat outside the view for more than 3 ticks |
| `fight-watchdog-rescue` | major | the character's own hit/getup script never returned control |
| `fight-camera-nan` | critical | the camera position or zoom became non-finite |
| `fight-engine-warning` | minor | a warning the engine raised during the fight (missing anim, missing state) |

## What the static pass checks

| Code | Severity | Meaning |
|---|---|---|
| `char-load-fail`, `char-load-panic` | critical | the pack cannot be loaded at all |
| `char-no-anims`, `char-no-states`, `char-no-commands` | critical | the `.air`/`.cns`/`.cmd` produced nothing |
| `char-web-quarantine` | critical | the browser import refuses the pack (`ume_formats::health`), so it never shows up on the web |
| `char-unsupported-controller`, `char-unknown-trigger` | major | a controller or trigger the engine does not run |
| `char-sprite-missing` | major | an animation frame references a sprite the `.sff` does not have |
| `char-sprite-transparent` | minor | the frame exists but every pixel is transparent |
| `char-sound-missing` | major | `PlaySnd` asks for a sample the `.snd` does not have |
| `char-unknown-parameter` | minor | a controller parameter the engine never reads |
| `char-wasm-atlas-overflow`, `char-wasm-atlas-pressure` | major/minor | the sprite pixels cannot fit the 2048 browser atlas (4.2 M pixels), so it has to evict and re-upload |
| `char-load-slow` | minor | loading took more than 2.5 s |

Stage codes: `stage-load-fail`, `stage-load-panic`, `stage-no-layers`, `stage-no-sprites`,
`stage-layer-sprite-missing`, `stage-layer-anim-missing`, `stage-bounds-tight`, `stage-spawn-overlap`,
`stage-music-outside-vfs`, `stage-no-shadow`.

## Reading the report

- `report.md` ranks the characters and stages with the worst findings, and lists the slowest loading packs and the
  slowest simulation ticks.
- `report.json` carries the summary, every finding and every fight (hits, damage, distinct states, tick cost).
- `summary.json` is the compact form (counts by code plus the first examples), small enough to keep in the repository.
- `issues.csv` is the same list flattened for a spreadsheet or a diff between two runs.

A finding is evidence to reproduce, not a verdict: `--only <name> --opponent <name>` replays the exact pair, and
`--no-fights` isolates the static problems of one pack.

## Web validation

The browser is the strictest target: it refuses packs that the desktop loader tolerates, and it has a much smaller
texture budget. The atlas findings come from summing the pixels of every sprite in the pack (indexed and RGBA alike)
and comparing with the 2048 atlas the wasm build uses (4.2 M pixels). Reproduce an actual import with:

```bash
node scripts/web-mugen-smoke.js web/dist mugen/incoming/my-char.zip
```

It imports the pack through the real file input, deep-links into a fight with the imported fighter and fails on a
panic, a frozen clock, a stalled renderer or a heap above the limit. When the pack is already in the roster (the
bundled `elecbyte.zip`), set `MUGEN_SMOKE_CHAR` to the fighter and the test validates content-key deduplication
instead of expecting a new entry.
