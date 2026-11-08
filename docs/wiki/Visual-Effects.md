# Visual Effects

Presentation-only effects live in `src/vfx.rs` and `src/vfx/`; the simulation never reads them.

- **Impact tiers** (`src/vfx/impact.rs`): light (<40 damage), medium (<90), heavy (heavy hitdef or >=90) and hyper (>=150). Each tier sets spark count and speed, glow, shockwave radius, flare, camera punch, shake, flash and chromatic kick. Works for any character because it reads the engine `Hit` event.
- **Ground cracks** (`src/vfx/crack.rs`): heavy and hyper hits on a grounded defender leave glowing cracks that fade out (high quality only).
- **Quality** (`src/vfx/quality.rs`): follows the `fx` option (0 low, 1 medium, 2 high). Counts scale by 0.35 / 0.7 / 1.0; medium drops cracks and stage lights, low also drops debris and half of the flare. Touch devices already start at medium.
- Particles stay in the shared `MAX_PARTS` pool; no allocation happens per hit beyond the pool.
- `UME_SHOTS_ONLY=fx` renders hit, heavy, hyper, counter, guard and super stills.
- **Super and awakening** (`src/vfx/awaken.rs`): layered rings (chest and ground), energy pillar of embers, 360 degree spark burst, ground cracks on high quality.
- **Dash** (`src/vfx/dash.rs`): speed streaks and a ground ring when a fighter starts a ground dash.
