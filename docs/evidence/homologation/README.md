# Homologation evidence

Run of `ume-tools homologate` over the local M.U.G.E.N collection (`mugen/chars`, `mugen/stages`, git-ignored) plus
the browser validation of the same packs in the wasm build.

Reproduce (the collection is not versioned, any folder of packs works):

```bash
cargo build --release -p ume-devtools
target/release/ume-tools homologate --out docs/evidence/homologation
node scripts/web-mugen-smoke.js web/dist <pack.zip>
```

## What the run covers

| metric | value |
|---|---|
| characters | 679 |
| stages | 127 |
| fights (CPU vs CPU, 30 s each, real stage background) | 1485 |
| tick budget measured | median 81 µs, p95 336 µs per tick |

`report.md` (kept here) ranks the roster and the stages; `summary.json` holds the counts by code; the full
`report.json` and `issues.csv` are ignored by git because they are several megabytes and are regenerated on demand.

## Problems this run found and fixed

| finding | evidence before | fix |
|---|---|---|
| `fight-panic` in 128 fights: `f32::clamp(10.0, -5.0)` inside `Match::update_camera` | 15 stage `.def` files ship `boundleft > boundright` (for example `Fake Karakura city`: `boundleft = 10`, `boundright = -5`) | `load_stage` sorts both camera ranges; new code `stage-bounds-inverted` reports the broken stage (`crates/formats/src/stage.rs`, `crates/formats/tests/stage_bounds.rs`) |
| phantom roster entries: stale `backup/` copies of 40 characters were listed as fighters, some with no animation at all | 719 "characters" before, 8 with `char-no-anims` | `find_char_defs`/`find_stage_defs` skip author backup folders (`crates/formats/src/vfs/find.rs`, test included) |
| `explod.angle` and `explod.alpha` were never read, so rotated and translucent explods rendered upright and opaque | 34 uses of `explod.angle` and 39 of `explod.alpha` in a single fighter | `Explod` carries both values to `DrawOpts` (`crates/engine/src/matchs.rs`, `crates/engine/src/matchs/types.rs`, `src/render.rs`, test in `crates/engine/tests/explods.rs`) |
| `PlaySnd volume` was ignored (only the Ikemen `volumescale` was read) | `playsnd.volume` reported as an unknown parameter on real packs | `volume` multiplies `volumescale` (`crates/engine/src/controllers.rs`) |
| the diagnostics probe reported supported parameters as unknown (`pause.movetime`, `helper.size.*`, `hitdef.palfx.*`, ...) | 19 false positives on one fighter | the probe now runs a second time with the parameters that gate a controller's early return (`crates/engine/src/diagnostics/params.rs`) |

After the fixes the same run reports **0 `fight-panic`** and the failing stages appear as `stage-bounds-inverted`.

## Findings that are pack defects (not engine defects)

- 8 characters have no animation at all (`char-no-anims`) because their `.air`/`.cns` are not next to the `.def`.
- 20 packs are quarantined by the browser importer for the same reason (`char-web-quarantine`): the web build refuses
  them, the desktop loader only tolerates them when a file with the same base name exists elsewhere in the pack.
- 2877 `PlaySnd` calls have no sample in the `.snd` and 3781 animation frames have no sprite in the `.sff`.
- 15 stages have inverted camera bounds, 25 have a fightable width under 120 px, 23 reference layer sprites that the
  stage `.sff` does not contain.

## Browser validation (the critical target)

`scripts/web-mugen-smoke.js` imports a real pack through the same `<input id="ume-file">` the player uses, deep-links
into a fight with the imported fighter and fails on a panic, a frozen clock, a stalled renderer or a heap above the
limit.

| run | result |
|---|---|
| import `Yuno.zip` (11.7 MB, 1 character) | roster 23 → 24, `wasm` 106 → 164 MiB, fight advanced tick 64 → 241 in 3 s |
| import `assets/mugen/elecbyte.zip` (bundled, deduplicated by content key) | roster unchanged, `kfm` still fights |
| native smoke (`scripts/web-smoke.js`) | `WEB SMOKE OK` |

Screenshots: `web/imported.png` (roster after the import), `web/fight-yuno.png` (imported Yuno fighting in Chromium),
`web/fight-kfm.png` (Kung Fu Man in the same build).

The browser test is part of the `web-smoke` CI job.
