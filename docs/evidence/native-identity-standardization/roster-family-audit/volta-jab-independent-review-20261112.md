# Volta jab family palette correction review — 2026-11-12

## Result

Reapplied the exact seed-13 shared palette to active `jab0.png`. Strict apply passed at max dE **0.056425** against a **0.13** gate. The updated image has zero output pixels outside the palette and zero partial-alpha pixels. All active `jab0`, `jab1`, and `jab2` colors are exact members of the same 48-entry palette (SHA-256 `25f9eaf37d1054396f7305e861a5f54ce0e5eb5395ac17f2a7231c213995434f`).

`jab0` is 400×400 RGBA, binary alpha, with 34 opaque colors plus transparency; axis `[200,392]`, `refHeight=320`, visible floor y=392. `jab1` and `jab2` retain their prior validated hashes and remain exact members of the same palette.

## Backup and provenance

- Previous active jab0 SHA: `719842ee823a2e639c51e6f6117d5824559d4aa35cfd26f7d9ad9d8734c8b8fd`
- Promoted jab0 SHA: `d51d88b3ae623e240fc854f0c89e12acc3248e6353a402dcd4a02ecf13f65a8c`
- Original PNG backup: `integrated/volta-jab-palette-reapply/backup/jab0.before-shared-palette.png`
- Strict QA: `integrated/volta-jab-palette-reapply/reapplied-run1/apply-qa.json`
- Promotion manifest: `integrated/volta-jab-palette-reapply/promotion-manifest.json`
- Prior native/2× sheets are preserved under `integrated/volta-jab-palette-reapply/backup/`.

## Visual and animation review

Updated labeled native and 2× nearest contacts include `04_special`, jab0 guard, jab1 impact, and jab2 recovery. Identity and pose progression remain legible; scale and floor line are consistent. A small contrast difference remains because jab1/jab2 use more of the lighter silver palette colors than jab0 and the canonical special frame.

The animation still runs jab0 (3 ticks) → jab1 (3) → jab2 (4) → idle0 (2), 12 ticks total. Hit metadata remains on jab1, element 2, and the move hit remains at element 2. No runtime or collision test was performed; those checks remain pending.
