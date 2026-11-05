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
