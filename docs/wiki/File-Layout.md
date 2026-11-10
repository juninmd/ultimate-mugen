# File layout

**English** · [Português (BR)](pt-BR/Layout-de-Arquivos.md)

A feature must be reviewable and mergeable without waiting for other features. That only works when a file has one reason to change: two people editing different features then touch different files, and a pull request that adds a mode does not rewrite the file another pull request is editing.

## The 250-line rule

- No source file exceeds **250 physical lines** (`node scripts/file-size.mjs`).
- The ceiling is not a target. Most files should stay between 80 and 200 lines: if a file already has two reasons to change at 120 lines, split it now.
- The rule covers Rust, JavaScript, TypeScript, CSS, HTML and WGSL - every file a developer reads as code.
- It does not cover content and docs: UMF JSON, `.cns`, `.air`, `.def`, `.snd`, sprite atlases, `.md` pages and generated images.
- Generated Rust tables count as source. When they do not fit, put them in a data-only file under a folder named for the table (for example `native/tables/`) so nobody edits them by hand.

## Grandfathering (the ratchet)

The tree already has files over the ceiling. They are listed in [`scripts/file-size-baseline.json`](../../scripts/file-size-baseline.json) with the exact line count they had when the rule landed.

- A file in the baseline may stay large, but it may not grow by a single line.
- A file that is not in the baseline must be born under 250 lines.
- A file that goes under 250 lines must leave the baseline. Run `node scripts/file-size.mjs --update` in the same pull request that split it.
- Never regenerate the baseline to raise a number. `--update` is only valid together with a split.

CI runs `node scripts/file-size.mjs`; a failing run prints each offending path with its count and the limit.

## One responsibility per file

A file has one responsibility when all the answers below are the same:

| Question | Same responsibility | Different responsibility |
|---|---|---|
| Why would it change? | two functions change when the format changes | one changes with the format, the other with the camera |
| What data does it own? | one struct and its helpers | several structs that talk to each other |
| Who reviews it? | the same reviewer with the same context | a different specialist |
| Can it be tested alone? | yes, with the crate's public API | only through the whole screen |

The file name states the responsibility: `collision.rs`, `hitdef.rs`, `palette.rs`, `lifebars.rs`. Names that hide a responsibility are rejected in review: `utils.rs`, `helpers.rs`, `misc.rs`, `common.rs`, `manager.rs`.

## Folder shape

A feature is a folder; the file next to it is its public face.

```
crates/engine/src/native/
  mod.rs            facade: declarations and re-exports only
  compile.rs        UMF JSON -> states
  pack.rs           sprite atlas packing
  rig.rs            skeleton types and the public entry points
  rig/
    body.rs         body joints
    detail.rs       hair and cloth joints
    face.rs         face joints
```

- `feature.rs` holds the types every part shares and the functions the rest of the crate calls.
- `feature/<concern>.rs` holds one concern each. Rust lets a folder sit next to the file with the same name, so no `mod.rs` is needed inside it.
- `mod.rs` is a facade: module declarations, `pub use` re-exports, nothing else. Aim for 60 lines; a `mod.rs` with logic is a file that was not split.
- Move a type with the code that owns it. Keep the old path working with a re-export (`pub use collision::ResolvedHit;`) so a split does not become a rename across the whole tree.
- The app mirrors the same idea: `src/app.rs` keeps the frame and state, `src/app/<screen>.rs` owns one screen each.

### `src/app/` screen map

`src/app.rs` keeps the `App` state, the per-frame dispatcher (`App::frame`) and the screens not split yet (character select, VS loading). Every other screen is one file, so a change to one screen never touches another:

| File | Owns |
|---|---|
| `mode.rs`, `screen.rs`, `team.rs` | `Mode`, `Screen` and `Team` types (re-exported from `app`) |
| `title_screen.rs`, `menu_screen.rs`, `menu_focus.rs` | Title, main menu, eased menu focus |
| `stage_select.rs` | Stage grid |
| `load_plan.rs`, `vs_screen.rs` | Who loads with which palette, VS screen, `start_fight` |
| `fight_screen.rs` | The fight frame and team rotation |
| `results_screen.rs` | Results screen |
| `netplay_session.rs` | Netplay start, overlay and replay capture |
| `fighter_draw.rs` | Fighter preview drawing shared by menus |
| `pause/`, `arcade/`, `story/`, `cutscene/` | Features that already had a folder |

A screen file is `use super::*;` plus one `impl App` block; its entry method is `pub(super)`. Add a screen by creating the file and wiring it with one `mod` line and one `match` arm in `App::frame`.

## Splitting an existing file

1. List the responsibilities in the file. If the list is not obvious, read the callers first; the file is not understood yet.
2. Create `feature/` and move one responsibility per pull request, one commit per extraction.
3. Change no behavior while extracting. Tests must pass before and after, and the diff must be mostly moves.
4. Extract the smallest cohesive unit first (the one with fewer callers). Big-bang rewrites conflict with every open pull request.
5. Update this page if the new layout is different from the one described here.

## Working in parallel

- Prefer new files. A feature adds `feat/<thing>.rs` and one registration line; it does not append a block to a hot file.
- Registration points are the only files two features legitimately share. Add one line, in alphabetical order, and never reformat the file: `controllers.rs` and `triggers.rs` match arms, `mod.rs` declarations, `assets/chars/index.json`, route tables.
- If a feature must touch a hot file, land the logic in its own file first and rebase the one-line wiring last, so the conflict is a single line.
- Do not rename or reformat a shared file in a feature pull request. Do it alone, with no behavior change.
- One wiki page per feature, linked from [Home](Home.md), so two features do not edit the same paragraph.

## Hot files to split first

These files hold the most unrelated responsibilities and are the current source of merge conflicts. Splitting one of them removes its baseline entry.

| File | Lines | Suggested split |
|---|---|---|
| `crates/engine/src/native/rig.rs` | 2265 | `rig/skeleton.rs`, `rig/pose.rs`, `rig/palette.rs` next to the existing `body.rs`, `detail.rs`, `face.rs` |
| `crates/engine/src/matchs.rs` | 1751 | `matchs/round.rs`, `matchs/collision.rs`, `matchs/camera.rs`, `matchs/tick.rs` |
| `src/app.rs` | 2238 | Nine screens already live in `src/app/`; next are `select` (830 lines, split by grid, side panel and input) and `load_step_inner` |
| `src/stages.rs` | 1069 | `stages/load.rs`, `stages/layers.rs`, `stages/motion.rs` |
| `src/stagefx.rs` | 704 | `stagefx/scenes.rs` already exists; extract `stagefx/particles.rs`, `stagefx/weather.rs` |
| `src/market.rs` | 700 | `market/install.rs`, `market/listing.rs`; the crate keeps the logic |
| `crates/engine/src/native/rig/detail.rs` | 723 | `detail/hair.rs`, `detail/cloth.rs` |
| `crates/formats/src/screenpack.rs` | 610 | `screenpack/lifebars.rs`, `screenpack/motif.rs` |

## Review checklist

- [ ] No new file over 250 lines.
- [ ] No grandfathered file grew.
- [ ] A file that crossed under 250 lines left the baseline in this pull request.
- [ ] New logic lives in a file named after it, not in `utils.rs`.
- [ ] Shared files show one-line registrations, no reformatting.
- [ ] The wiki page for the feature is updated in English and Portuguese.

## Related pages

- [Customizing the engine](Engine.md) - architecture, adding a controller, a trigger or a format.
- [Performance](Performance.md) - where the hot paths live.
