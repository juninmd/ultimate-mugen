# ADR 0007: UMF 2 is the definitive native fighter format

- Status: Accepted
- Date: 2026-11-21

## Context

Native fighters were one minified JSON line of about 50 KB each (21 fighters, 1.04 MB). Reviews could not read the diffs, frame lists repeated `{"pose": ..., "ticks": ...}` hundreds of times, and nothing stated what a fighter must contain. Formats compared: TOML, RON, YAML, KDL, JSON5, M.U.G.E.N CNS, ZSS and binary. None beats JSON here: serde and wasm already read it, every editor and CI tool understands it, and the compiler already turns it into the shared bytecode (ADR 0002).

## Decision

Keep JSON and define UMF 2:

1. `"format": "umf/2"`. `umf/1` files still load, so marketplace and imported fighters keep working.
2. A frame with only a pose name and a tick count is written `"pose*ticks"`. Frames with `hit`, `effect`, `sprite`, `hurt` or `offset` stay objects. Both forms can mix in one list.
3. One canonical layout, produced by `node scripts/umf2-migrate.mjs`: stable key order, values on one line up to 200 columns, objects and arrays expanded above that. CI fails when a shipped fighter is not canonical.
4. A quality floor tested in `native_quality_bar`: every fighter keeps at least 17 normals, 9 specials, 3 supers and 3 projectile moves, no zero-tick frame and no hit without damage.
5. The migration is lossless: the deserialized fighter of all 21 shipped files is identical before and after, apart from the `format` field.

## Consequences

Fighters stay one file at `assets/chars/<id>.json`: the roster tests, examples and tools address that path and splitting it would touch about 100 files for little gain. Per-fighter folders, shared move kits beyond `kits/core.json` and a packed release blob remain open and need their own ADR if they are wanted. The migrator is idempotent, so future edits only need to run it before committing.
