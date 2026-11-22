# UMF schema

`docs/schema/umf-2.schema.json` is the machine-readable definition of [UMF 2 fighters](UMF-Character.md) and `umf-kit/1` kits. It is a JSON Schema (2020-12) that mirrors the serde structs in `crates/native/src` and **rejects unknown keys**, so `"hitz"` or `"lyfe"` fails instead of being ignored by the loader.

## Check a fighter

```bash
node scripts/umf-schema.mjs                       # every fighter, kit and example in the repository
node scripts/umf-schema.mjs path/to/fighter.json  # one file, e.g. a pack from mugen-assets
```

The validator is in-repo (`scripts/umf-schema.mjs`, no dependency). CI runs it through `scripts/umf-schema.test.mjs` in the `release-scripts` job, so a shipped fighter that breaks the schema fails the pull request. Errors name the JSON pointer: `assets/chars/kaito.json: /moves/3/hits/0/level: "mid" is not one of ["light","medium","heavy"]`.

## Use it in an editor

Add `"$schema"` to the fighter (the loader ignores it) or map the file in your editor settings. VS Code:

```json
{ "json.schemas": [{ "fileMatch": ["assets/chars/*.json", "umf/**/*.json"], "url": "./docs/schema/umf-2.schema.json" }] }
```

## What it covers

| Area | Definition |
|---|---|
| Identity and profile | `id` (`^[a-z0-9][a-z0-9_-]*$`), `profile` with `en`, `pt`, `es` texts, `element`, `alignment` |
| `stats`, `passives`, `abilities` | Typed numbers, `fly`, `teleport`, `superSpeed`, `charge`, `forms` |
| `anims` | `frames` as `"pose*ticks"` strings or objects with `pose`, `sprite`, `effect`, `hit`, `hurt`, `offset`, `ticks` |
| `moves` | `kind`, `when`, `hits`, `projectile` (one or list), `cancels`, `rekka`, `throw`, `cinematic`, `explods`, `afterimage`, `shake`, `flash`, `superPause`, `followUp`, `blink`, `vfx`, `quake`, `velMul`, `velAdd`, `posFreeze`, `posSet`, `palFx` |
| `helpers`, `bossRules`, `scripts` | Stand, clones, boss phases, ZSS and CNS hooks |
| Kits and `extends` | A fighter with `extends` or `format: "umf-kit/1"` may omit the required fields; a move then needs only its `id` |

`art.poses` and `art.style` are open objects: they belong to the sprite pipeline and the deprecated rig. The legacy top-level `stage` and `country` are allowed and ignored.

## What it does not check

References between parts (an `anim` name that exists, a `cancels` id that exists, a PNG that exists) and the balance floor. Those stay in the Rust tests (`native_quality_bar` and the roster tests).

## Changing the format

A change to a serde struct in `crates/native/src` must change the schema in the same pull request. Run `node --test scripts/umf-schema.test.mjs`: it fails when the schema rejects a shipped fighter, which is how a field added to the loader but not to the schema shows up.

## Storage format

JSON for definitions and PNG for sprites were measured against MessagePack, CBOR, bincode, postcard, QOI, lossless WebP, LZ4 and atlases: see [ADR 0008](adr/0008-umf-schema-and-storage-review.md).
