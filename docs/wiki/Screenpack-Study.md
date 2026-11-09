# Screenpack study

What M.U.G.E.N and Ikemen GO screenpacks (motifs) do differently from a plain menu, and what Ultimate MUGEN took from it. Only ideas are used; no third-party art, layouts or sounds are copied.

## Sources

- [Ikemen GO wiki: Screenpack features](https://github-wiki-see.page/m/ikemen-engine/ikemen-go/wiki/Screenpack-features/)
- [Ikemen GO wiki: Updating to 1.0](https://github.com/ikemen-engine/Ikemen-GO/wiki/Updating-to-1.0)
- [Ikemen GO menu system (DeepWiki)](https://deepwiki.com/ikemen-engine/Ikemen-GO/6.2-menu-system)
- [Ikemen GO issue 329: KOF98-style versus select](https://github.com/ikemen-engine/Ikemen-GO/issues/329)
- [MUGEN Guild: screen pack docs](https://mugenguild.com/forum/topics/screen-pack-docs-178231.0.html)
- [MUGEN Guild: X-Men Children of the Atom screenpack for Ikemen GO](https://mugenguild.com/forum/topics/xmen-children-atom-screenpack-version-1-1-for-ikemengo-system-199912.0.html)
- [MUGEN Guild: Street Fighter Alpha Zero screenpack, 16:9 and 4:3](https://mugenguild.com/forum/prev_next.next/topics/street-fighter-alpha-zero-agent-ed-v-1-3-198589.0.html)

The search was limited to forum threads and wiki mirrors; the default motif files themselves could not be fetched, so parameter names below are from secondary sources.

## What packs do differently

| Area | Differential | Ultimate MUGEN |
|---|---|---|
| Structure | One `system.def` points to `select.def`, `fight.def`, sprites, sounds and fonts; any missing file falls back to the default motif | Same layout and fallback, see [Screenpacks](Screenpacks.md) |
| Select | Grid size and cursor set by parameters, animated portraits, per-character stage and slots | Grid, cursors, team slots and palette row already native, see [Character select](Character-Select.md) |
| Versus | Order select on the versus screen, animated portraits, KOF98-style pick | Versus screen with team strips exists; order select is a follow-up |
| Victory | Winner portrait, name and a typed victory quote after Arcade, Versus, Quick Match and Watch | Winner pose and name existed; quote is a follow-up |
| Results | Time, combo and damage grading, time-challenge best times on the select screen | New stats strip (below) |
| Resolution | Packs ship for 4:3 and 16:9 with `localcoord` | `localcoord` is converted to the 426x240 HUD space |
| Parser | Unknown fields produce warnings instead of silent failure | Checker prints one `warning:` per broken file |

## Implemented from this study

The results screen shows four glass chips under the winner line: match time, best combo of each side, total damage dealt by each side and life left of each side as a percentage. `src/match_stats.rs` observes the match after every tick (outside the deterministic engine, so rollback and replays are unaffected) and `src/app/results_stats.rs` draws the chips. Texts are localized in English, Portuguese and Spanish.

Preview: `UME_SHOTS_ONLY=results xvfb-run -a target/release/ultimate-mugen --shots <dir> --res 1280x720 --mute`.

## Next candidates

1. Order select inside the versus screen for team modes.
2. Victory quote line typed under the winner name.
3. Rank grade (S to D) from time, life left and combo.
4. Best-time records per character in the select screen for time-attack modes.

## Deeper survey (second pass)

Community screenpacks checked by their stated features (no art or layout copied):

| Pack | Differential | Takeaway for the engine |
|---|---|---|
| [IKEMEN Generations HD (OroCrimson)](https://www.andersonkenya1.net/files/file/23994-ikemen-generations-hd-screenpack-by-orocrimson-ikemen-go-099/) | Category-first select: up/down picks a category, left/right a character; custom KOF 2002 style lifebars | Category navigation for big rosters |
| [Infinite Legacy X HD](https://mugenfreeforall.com/topic/49257-infinite-legacy-x-screenpack-for-ikemen-go-hd-1280-x-720-by-orocrimson-updated-january-2025/) | Very large slot counts (700 slots); author notes the engine struggles with huge grids | Paging and virtualized grid already exist; keep it that way |
| [Capcom vs SNK HD Black Edition](https://mugenfreeforall.com/topic/48495-capcom-vs-snk-hd-black-edition-for-ikemen-go-hd-1280-x-720-099-updated-05032024/) | Selectable slot densities (120 to 840) | Grid density as a setting |
| [Anything VS Anything](https://www.andersonkenya1.net/files/file/42054-anything-vs-anything-ikemen-go-screenpack/) | 999 slots navigated only up and down; users report it is slow to pick | Confirms need for jump keys (PgUp/PgDn already in place) |
| [Ikemen-GO-Screenpack](https://github.com/ikemen-engine/Ikemen-GO-Screenpack) and [PR #56](https://github.com/ikemen-engine/Ikemen-GO-Screenpack/pull/56) | Official default motif being replaced with original art and MIT fonts | Reference for a clean default motif |
| [Engine PR #4146](https://github.com/ikemen-engine/Ikemen-GO/pull/4146) | Alpha and blend on every animation and text element; slot cycling; stage select buttons | Per-element alpha is worth copying in our HUD tokens |
| [Ikemen-GO-Plus screenpack.lua](https://github.com/Windblade-GR01/Ikemen-GO-Plus/blob/K4-experimental/script/screenpack.lua) | Lua hooks read localcoord from `system.def` | Scripting is out of scope; keep data-driven |
| [external-lua-modules](https://github.com/doomjoshuaboy/external-lua-modules) | Carousel-style select as external modules | Candidate for a future carousel layout |

Also noted: the grid in recent Ikemen nightlies is oblique and capped near 80 slots by default, which suggests a single fixed grid model is a weak point for the whole ecosystem.

Caveats: several pages are older (2023 to 2025), and popularity is inferred from forum and listing pages, not download counts.
