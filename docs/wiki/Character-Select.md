# Character select

## Research summary

Common traits of well-known fighting-game select screens (Street Fighter, Tekken, King of Fighters, Guilty Gear, Marvel vs. Capcom, Smash):

- A compact roster grid in the centre, with a highly visible cursor and the hovered fighter shown large on the side of the player who owns the cursor.
- Each player has a distinct colour (blue for P1, red for P2) used for the cursor, the name plate and the panel edge.
- The big portrait changes instantly on hover with a short slide and fade, never a hard pop.
- Confirming gives an immediate, clearly different response: flash, pose change, a "ready" mark and a sound.
- Team modes show a row of slots that fill in order, with the next slot highlighted.
- The stage is chosen on the same screen or right after it, with a thumbnail and name.
- Every fighter is shown at a comparable on-screen size so small characters are not lost.

## Design adopted

- Glass panels, the same look as the menu and HUD: translucent body, light top edge, accent edge per side.
- Left and right panels with the hovered fighter in a large frame, a coloured glow from the fighter's own colour, a floor shadow, slide and fade on change, a flash and ring on confirm and the confirmation pose.
- Smoothly following cursor with pulsing fill, corner brackets and a P1/P2/CPU tag; non-hovered tiles are dimmed.
- Team slots (up to 5) with portrait, name and a numbered badge on the picked tiles.
- Stage chip in the header, cycled with `[` and `]` or by clicking the arrows; the stage grid still confirms it after the fighters.
- Palette row under each side panel: 12 numbered swatches with the current one highlighted. `,` / `.` cycle, `P` picks a random palette, clicking a swatch jumps to it; each side keeps its own palette and team members are stored independently. The last palette of each player is persisted in `ultimate-mugen.cfg` (`pal1`, `pal2`) and in replays.
- Keyboard, gamepad and touch/mouse use the same navigation as before.
- Grid navigation moves by column: `Up` and `Down` keep the current column and land on the closest tile of the target row, so a shorter last row never blocks or skips the cursor. `PgUp`/`PgDn` move a whole page, `Home`/`End` jump to the ends, the wheel scrolls by row and `R` picks a random tile.

## Uniform character size

`draw_char_fit` scales a fighter so the union of the bounding boxes of all idle-animation frames fits a fixed frame, centred horizontally with the feet on the baseline. The extent is computed once per fighter and cached (`App::char_extent`). The same function is used by the select panels, the roster tiles that fall back to sprites, and the versus screen. Sprite and stage art files are never modified.

## Performance

- Only visible tiles are drawn and decoded; one portrait is decoded per frame.
- Upper-case names and the stage/filter labels are cached or written into a reused buffer, so the screen allocates nothing per frame in steady state.

## Strength, life and sorting

Each tile has a thin gold bar at its bottom edge for strength. The side panel shows three gauges above the name plate: FOR (strength), VID (life) and VEL (walk speed).

Definitions, read from each character's data (native and M.U.G.E.N alike):

- **Strength** = `data.attack` × `data.life` / 1000 (defaults 100 and 1000, so an average character is 100). Gauge full at 200.
- **Life** = `data.life` (default 1000). Gauge full at 2000.
- **Speed** = `velocity.walk.fwd.x` (default 2.4). Gauge full at 6.0.

Press `O` on the select screen to cycle the order: roster, strength, life, speed, name. The choice is saved as `sort=` in the config. Stats come from the character's cached preview and are computed once per character.

Limitation: sorting by a stat loads the preview of every character in the view the first time, so a very large M.U.G.E.N roster can take a moment on that first press.

## Large rosters (1000+ fighters and stages)

- The visible list (filter, search and sort) is computed once and reused until the filter, sort, search text or the playable set changes (`src/app/select_view.rs`). With 1017 fighters a name sort costs about 0.9 ms when rebuilt and 0.08 ms when reused.
- The grid only uses as many rows as fit above the footer, or above the team slots in team modes, so the slots never cover the last row (`src/app/select_grid.rs`).
- The cursor tag (P1, P2, CPU) is drawn inside the tile, so it no longer covers the name of the tile above. Tile names are cut to the tile width with an ellipsis (`App::fit_label`).
- A search pill under the title shows the current search and the number of results; `/` or a click opens the search page, `Esc` or a click clears it. Search matches name, id, author and series. On the search page the arrows and `PgUp`/`PgDn` choose a result and `Enter` moves the cursor to it.
- The footer shows the page (`page 3/15`) next to the position.
- `F` or the star button marks the fighter under the cursor as a favorite; favorites show a gold star on their tile and are saved (`favorites.txt`). Every pick is recorded in the 24 most recent picks (`recent.txt`). The FAVORITES and RECENT filter chips show only those fighters, recent ones newest first (`src/app/select_marks.rs`).
- The author chip (`U`, `Shift+U` or a click) cycles through authors ordered by fighter count and combines with the other filters (`src/app/select_author.rs`).
- On touch devices the search page shows an on-screen keyboard (letters, digits, space, delete, OK, close) and fewer result rows; tapping a result picks it (`src/app/select_search/keyboard.rs`). `UME_OSK=1` forces it on desktop.

## Stage select

- Filter chips (All, Originals, M.U.G.E.N), cycled with `Tab`, and a search box opened with `/` or a click; `Esc` clears the search.
- Names are cut to the tile width with an ellipsis; `Home`/`End` jump to the ends; the footer shows position and page.
- `Random` picks among the stages that match the current filter and search.
- M.U.G.E.N stage previews start only after the cursor rests for 0.12 s, one per frame, and previews more than six rows away are released, so scrolling through hundreds of M.U.G.E.N stages does not keep every decoded stage in memory.

## Stress mode

`UME_STRESS=<n>` (desktop) or `?stress=<n>` (web) adds `n` generated fighters and `n` generated stages (up to 5000) that reuse the bundled content with generated names, authors and series. It is a test hook only. `UME_STRESS=1000 UME_SHOTS_ONLY=roster ultimate-mugen --shots <dir> --res 1280x720 --mute` prints `roster-perf:` lines (view cost per sort, CPU time per frame while walking the grid and the stage list, memory) and saves screenshots of the grid, search and stage filters.
