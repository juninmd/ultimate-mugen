# Visual Effects

Presentation-only effects live in `src/vfx.rs` and `src/vfx/`; the simulation never reads them.

- **Impact tiers** (`src/vfx/impact.rs`): light (<40 damage), medium (<90), heavy (heavy hitdef or >=90) and hyper (>=150). Each tier sets spark count and speed, glow, shockwave radius, flare, camera punch, shake, flash and chromatic kick. Works for any character because it reads the engine `Hit` event.
- **Ground cracks** (`src/vfx/crack.rs`): heavy and hyper hits on a grounded defender leave glowing cracks that fade out (high quality only).
- **Quality** (`src/vfx/quality.rs`): follows the `fx` option (0 low, 1 medium, 2 high). Counts scale by 0.35 / 0.7 / 1.0; medium drops cracks and stage lights, low also drops debris and half of the flare. Touch devices already start at medium.
- Particles stay in the shared `MAX_PARTS` pool; no allocation happens per hit beyond the pool.
- `UME_SHOTS_ONLY=fx` renders hit, heavy, hyper, counter, guard and super stills.
- **Super and awakening** (`src/vfx/awaken.rs`): layered rings (chest and ground), energy pillar of embers, 360 degree spark burst, ground cracks on high quality.
- **Dash** (`src/vfx/dash.rs`): speed streaks and a ground ring when a fighter starts a ground dash.
- **KO finish** (`src/vfx/ko.rs`): three staggered shockwaves, white core, 360 degree sparks, radial slashes, debris and ground cracks on the loser. Simulation-side cost: `cargo test --release --bin ultimate-mugen cost -- --ignored --nocapture`.

## Cinematic post-processing (`assets/postfx.toml`)

`src/postfx/look.rs` reads `postfx.toml` (next to the executable, or `localStorage` on the web) and falls back to the embedded `assets/postfx.toml`. Missing keys keep neutral values and out-of-range values are clamped.

| Section | Keys | Effect |
|---|---|---|
| `bloom` | `intensity`, `threshold` | Glow strength and the brightness where it starts |
| `grade` | `tonemap`, `exposure`, `split`, `shadows`, `highlights` | ACES filmic curve (0 off, 1 full), exposure, and a split tone that tints shadows and highlights |
| `distortion` | `shockwave`, `heat` | Screen refraction from impact rings and heat haze around supers |
| `lens` | `chroma`, `vignette`, `grain` | Multipliers for the existing lens effects |
| `quality` | `grade_from_level`, `shockwave_from_level`, `heat_from_level`, `bloom_from_level` | Lowest `fx` level (0 to 2) where each effect runs |

- Shockwaves: rings with a radius of 24 or more push up to 4 refraction bands (`ume_shell::waves::shockwave`) into the composite pass; no extra render pass.
- All of it lives in the single composite shader, so the cost is one extra texture-free math block per pixel; `fx` level 0 skips post-processing entirely.
- Tests: `cargo test -p ume-render --test headless` and `cargo test --bin ultimate-mugen postfx`.
