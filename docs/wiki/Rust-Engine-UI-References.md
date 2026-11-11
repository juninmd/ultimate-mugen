# Rust engine UI references

Study of the eleven Rust engines listed in "Best Rust Game Engine for Game Development" (WeeTech Solution, 2026) and what is worth borrowing for menus, loading screens, HUD, settings and buttons. Nothing is copied into this repository. Only ideas and layout patterns are recorded here.

Stack constraints that bind every recommendation: wgpu + winit renderer, tiny-skia, custom immediate-mode UI in `src/ui.rs` and `src/app/menu_view.rs`, layout tokens in `src/ui_tokens.rs`. No macroquad, no egui, no Lua.

## Method and limits

- Source was read only for five repositories, shallow-cloned at the session's read access: `not-fl3/macroquad` (MIT OR Apache-2.0), `ggez/ggez` (MIT), `FyroxEngine/Fyrox` (MIT), `bevyengine/bevy` (MIT OR Apache-2.0), `AmbientRun/ambient` (MIT OR Apache-2.0). Licence text was confirmed from the repository files.
- The other six (Amethyst, Piston, Nannou, Blue Engine, Tetra, rend3) were not opened. Their notes below come from the article only and are marked as unverified.
- The article itself is marketing text. Its descriptions are generic and some claims (for example, which engines are actively maintained) were not checked.

## Engine by engine

| Engine | Licence | UI relevance | What we can use |
|---|---|---|---|
| Macroquad | MIT OR Apache-2.0 | High. Immediate-mode UI in `src/ui/` (widgets, `style.rs`, `key_repeat.rs`, `combobox`, `clipboard.rs`, `input_handler.rs`) | Closest model to our UI. Key-repeat for held arrows in menus, a style/skin struct for button and panel colours, combobox behaviour for settings. Stack rule: we do not adopt the crate. |
| Fyrox | MIT | High. `fyrox-ui` is a full retained-mode widget set: buttons, dropdown list, dropdown menu, tab control, scroll viewer, progress bar, message box, popup, file browser, navigation, style | Reference for the set of widgets a settings screen needs, focus `navigation.rs` for keyboard and gamepad, `progress_bar` for loading, `messagebox` for confirm dialogs, and a style module for themes. Retained-mode message passing does not fit our immediate-mode code, so ideas only. |
| Bevy | MIT OR Apache-2.0 | Medium. `bevy_ui` (flexbox), `bevy_ui_render`, `bevy_ui_widgets` (headless widgets, behaviour separated from styling) | The split between widget behaviour and visual style is worth copying conceptually. Flexbox is not needed; our 1280x720 virtual layout with tokens is enough. Bevy's state-based screen flow suggests how to structure loading and transitions. |
| Ambient | MIT OR Apache-2.0 | Low to medium. A UI element tree (`element`, `element_component`) built on its own runtime | Only the idea of composing small reusable elements (button, panel, label) with props. Its runtime is too large to borrow from. |
| GGEZ | MIT | Low. No UI layer; has a graphics text/glyph path and timers | Reference for a minimal loop and text rendering with glyph caching. Nothing to copy for UI chrome. |
| Tetra | not checked | Low (article only) | Nothing needed beyond what we have. |
| Amethyst | not checked | Low (article only). Project status should be checked before any reference | Skip. |
| Piston | not checked | Low (article only) | Skip. |
| Nannou | not checked | Low (article only; creative coding, not a game UI) | Skip. |
| Blue Engine | not checked | Low (article only) | Skip. |
| rend3 | not checked | Low (renderer only, no UI) | Skip. Its post-processing ideas are covered by our own `postfx`. |

## Prioritised recommendations

Ordered by player-visible impact and by how little they cost to build on the current UI code.

1. **Loading screen with a progress bar and tips.** Fyrox's `progress_bar` and Macroquad's simple panel style show the pattern. We already have `src/loading_tips.rs`; add a determinate bar driven by the real load stages (parallel load already exists, see [Performance](Performance.md)). Expected result: loading no longer looks frozen.
2. **Keyboard and gamepad focus navigation with key-repeat.** Macroquad's `key_repeat.rs` and Fyrox's `navigation.rs`. Holding up/down in menus should step smoothly, and focus should be visible on every screen.
3. **Settings widgets: dropdown, tabs, scroll viewer.** Fyrox's `dropdown_list`, `tab_control`, `scroll_viewer` and Macroquad's combobox define the expected behaviour (open, close on outside click, keyboard selection). Put them in new files under `src/ui/` or `src/app/`, using `ui_tokens`.
4. **Confirm dialogs and tooltips.** Fyrox's `messagebox` and `popup` for "quit match?", "delete replay?" and similar prompts. Keeps destructive actions explicit.
5. **Theme struct for colours and sizes.** Macroquad's `style.rs` and Fyrox's `style` module. We already have 11 themes in `menu_art.rs`; moving their values into one `Theme` struct with tokens makes adding a theme a data change.
6. **Screen transitions as a small state machine.** Bevy's state pattern. Fade or slide between title, select, fight and results, with easing matching the existing menu entry animation (ease-out cubic, 0.28 s).
7. **Headless widget behaviour.** Bevy's `bevy_ui_widgets` idea: separate hit-testing and focus logic from drawing so the same button logic serves the menu, pause and settings screens.

## What we will not do

- Adopt macroquad, egui or any engine's runtime. That breaks the stack decision in [Engine](Engine.md).
- Copy code from MIT or Apache-2.0 sources into the repo. Use them as design references only, so no licence notice is needed.
- Touch character sprites or stages. Those are handled elsewhere.

## Related

- [Engine](Engine.md) for the render and UI stack.
- [Visual Effects](Visual-Effects.md) for the effects pipeline.
- Portuguese version: [Referencias-UI-Engines-Rust](pt-BR/Referencias-UI-Engines-Rust.md).
