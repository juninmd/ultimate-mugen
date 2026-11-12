# Volta jab palette reapply — 2026-11-12

## Result

Reapplied the exact seed-13 shared palette (`25f9eaf37d1054396f7305e861a5f54ce0e5eb5395ac17f2a7231c213995434f`) to active `jab0.png`. Strict apply passed: max dE **0.056425** against a **0.13** gate; output has zero off-palette and zero partial-alpha pixels. The promoted jab0 is now 400×400 RGBA, binary alpha, floor y=392, 34 opaque colors plus transparency. All three active jab frames are exact RGB subsets of the same 48-entry palette.

## Provenance

- Previous active jab0 SHA-256: `719842ee823a2e639c51e6f6117d5824559d4aa35cfd26f7d9ad9d8734c8b8fd`
- Reapplied active jab0 SHA-256: `d51d88b3ae623e240fc854f0c89e12acc3248e6353a402dcd4a02ecf13f65a8c`
- Source backup: `integrated/volta-jab-palette-reapply/backup/jab0.before-shared-palette.png`
- Strict QA: `integrated/volta-jab-palette-reapply/reapplied-run1/apply-qa.json`
- Promotion manifest: `integrated/volta-jab-palette-reapply/promotion-manifest.json`

Previous contact sheets were preserved under `integrated/volta-jab-palette-reapply/backup/`. Updated native and 2× nearest contacts include the `04_special` identity anchor and all three jab poses at `integrated/volta-jab-before/volta-jab-integrated-native.png` and `...-2x-nearest.png`.

## Review state

The jab0 → jab1 impact → jab2 recovery progression remains readable. `jab1` still carries the hit on element 2; animation timing remains 3+3+4+2 = 12 ticks. A small tonal difference remains: jab1/jab2 armor plates read brighter than jab0 and the canonical special pose despite exact shared palette membership. No runtime validation was performed; runtime/collision checks remain pending.
