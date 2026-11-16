# Kaname attack-family cross-review — 2026-11-12

## Scope and evidence

Reviewed Kaname's attack families against `assets/chars/sprites/kaname/04_special.png` (identity authority) and `01_idle.png` (supplementary). Compared the existing `kaname-sprites.png` full-pack sheet, created focused 400×400 and nearest-neighbor 2× comparisons, and inspected key source frames individually. The 50 unique PNGs referenced by the selected report rows all exist and match their recorded SHA-256 values. No active sprite or character data was changed.

Evidence:

- [Basic attacks, native](kaname-attack-family-review/kaname-basic-native.png) and [2×](kaname-attack-family-review/kaname-basic-2x.png)
- [Special and super aliases, native](kaname-attack-family-review/kaname-special_aliases-native.png) and [2×](kaname-attack-family-review/kaname-special_aliases-2x.png)
- Existing [Kaname full sprite sheet](kaname-sprites.png)
- The machine-readable per-family rows, ordered frames and SHA-256 values are in `kaname-attack-family-review-20261112.json`.

## Confirmed findings

The canonical design is the blue-haired swordsman in the white long-sleeved haori with navy trim, black sash, navy pleated hakama, sandals and canonical katana. Many attack files instead show an older short navy/cyan outfit, covered/shorter hair silhouette, cyan belt and oversized cyan blade. This is not explained by a declared costume switch: `kaname_ascended` only recolors aura/energy, while the awakened form adds moves without a body/sprite-set swap. The effects around `04_special.png` are an identity reference only and are not to be copied into normal attacks.

| Family | Current report | Cross-review | Active source frames to review |
|---|---|---|---|
| jab | correction_required | confirmed | `jab0.png`, `jab1.png`; preserve canonical `idle0.png` return |
| strong | correction_required | confirmed | `str0.png`, `str1.png`; reused by several moves |
| lkick / hkick | correction_required | confirmed | `lk0.png`, `lk1.png`, `hk0.png`, `hk1.png`, `hk2.png` |
| cjab / clk / sweep | correction_required | confirmed | `cjab0.png`, `cjab1.png`, `crouch.png`, `crouch1.png`, `clk.png`, `sweep0.png`, `sweep1.png` |
| ajab / akick | correction_required | confirmed | `jumpTop.png`, `ajab.png`, `akick0.png`, `akick.png`, `jumpDown.png` |
| lowKick | correction_required | mixed family | `crouch0.png`, `crouch1.png`, `crouch.png` are old; preserve `lowKick.png`, which is canonical |
| airLowKick | correction_required | mixed family | `akick0.png`, `akick.png`, `jumpDown.png` are old; preserve `airLowKick.png`, which is canonical |
| stringA / stringB / stringC | correction_required | confirmed | Reuse the old jab, kick and strong frames listed above |
| drawThrow | correction_required | confirmed | `cr1.png`, `cr2.png`, `str0.png`; canonical `idle0.png` return |

## Static-pass rows contradicted by native review

The current family report marks these 12 attack families `static_pass`; their active sources visibly retain or alternate into the older outfit, so they need correction review:

| Family | Active old-design files and alias references |
|---|---|
| frostArc / frostArcEx | `cr0.png`, `cr1.png`, `cr2.png`; each sequence also contains the correctly dressed `inline_3f649693d432dce5.png`, so the family alternates outfits |
| blinkCut | `ss0.png`, `ss1.png`, `ss2.png` |
| iai / thousand / thousandUlt | Shared older `cr0.png`, `cr1.png`, `cr2.png` frames |
| trap | `mr0.png`, `mr1.png`, `mr2.png` |
| super | `sp0.png`, `sp1.png`, `cr2.png` alternate with correctly dressed `inline_325beea3d661fe78.png` and `inline_f4ead780ee459475.png` |
| needleBeam | `inline_2cdd134a69aabfbf.png`, `inline_4a5b82897bece4b5.png`, `inline_282e2507e149aa22.png` |
| crimsonCannon | `inline_f4f80989ec6dd44f.png`, `inline_385ffe023b743a7d.png`, `inline_6024007266145d23.png` |
| flashStep | `inline_4362387def07ae7a.png`, `inline_4642bd6b75a305a8.png`, `inline_d580a69b7cf04c8d.png` |
| petalStorm | `sp0.png`, `sp1.png`, `cr2.png`, `inline_b3540c4f422f879c.png`, `inline_7bce21d00d833441.png` |

The old-design inline files are still referenced by declared move frames; they are not unreferenced files. Some inline aliases in `frostArc` and `super` already use the canonical outfit. Do not replace all assets in a family indiscriminately.

The existing report already flags `hoarfrost` and `hoarfrostEx` as `correction_required`; their `mr0.png`/`mr1.png`/`mr2.png` and `jumpDown.png` sources match the old design. Four projectile/hit families (`frostArcFx`, `frostHit`, `winterFangFx`, `winterFangHit`) remain `unresolved`; these are effects, not body sprites.

## High-impact shared references and preserved frames

`cr0.png`/`cr1.png`/`cr2.png` are shared across frostArc, its EX, Iai, Thousand Frost and its ultimate; `cr2.png` also appears in `super` and `petalStorm`. `mr0.png`/`mr1.png`/`mr2.png` are shared by hoarfrost, its EX and trap. Fixing the shared sources will address several families, while retaining each distinct pose.

Native-size review confirms these individual sources already match Kaname's canonical design and should be preserved: `idle0.png`, `lowKick.png`, `airLowKick.png`, `inline_3f649693d432dce5.png`, `inline_325beea3d661fe78.png`, and `inline_f4ead780ee459475.png`.

## Decision

Prioritize replacing the old-design shared frames and active inline aliases while keeping the valid canonical frames and effect-only families separate. This is a visual source review; animation runtime remains untested. The initial cross-review corrected 12 false static-pass labels. The addendum below maps active moves, pose order, candidate state and priorities to the current character-data snapshot.

## Addendum — correction inventory and active move mapping

This inventory maps every attack family with `nativeReviewStatus=correction_required` to the current `assets/chars/kaname.json` move IDs and ordered pose names. “Correction required” means the sources show the older navy/cyan design or alternate it with the canonical costume. It does not mean every listed frame should be replaced by one image: keep action phases distinct and preserve canonical aliases. The current data snapshot contains 29 correction-required families, including mixed and currently unlinked files.

### Priorities and reusable source work

1. **Shared draw/special poses:** `cr0.png`, `cr1.png`, `cr2.png` span `frostArc`, `frostArcEx`, `iai`, `thousand`, `thousandUlt`; `cr2.png` also appears in `super`, `petalStorm`, `drawThrow`, `needleBeam` and `crimsonCannon`. `sp0.png`/`sp1.png` are shared by `super` and `petalStorm`. Correct each distinct pose once and update all active aliases; retain canonical inline frames and do not copy special-only effects.
2. **Shared normals/cancels:** `str0.png`/`str1.png`, `jab0.png`/`jab1.png`, `lk0.png`/`lk1.png`, and `hk0.png`/`hk1.png`/`hk2.png` feed normal, string, throw and cancel moves. Create distinct canonical candidates per phase before rewiring aliases.
3. **Shared hoarfrost/trap poses:** `mr0.png`/`mr1.png`/`mr2.png` serve `hoarfrost`, `hoarfrostEx` and `trap`; retain preparation, hit and recovery as separate frames.
4. **Dedicated aliases:** review `ss0.png`/`ss1.png`/`ss2.png` and old-design inline poses in `needleBeam`, `crimsonCannon`, `flashStep`, and `petalStorm`. Keep already canonical inline poses in `frostArc` and `super`.
5. **Low kicks:** both files match Kaname’s canonical outfit. Although neither is serialized in `assets/chars/kaname.json`, `native::low_normals::complete` injects ground and air low kicks after JSON deserialization from the existing `clk` and `akick` moves. The ground sequence resolves `crouch0 → lowKick → crouch1 → crouch`; the air sequence resolves through `airLowKick` with jump recovery frames. The engine roster test covers move injection and three distinct images; it remains unrun in this art pass. Both full families still need visual correction because surrounding crouch/air poses use the older outfit.

### Family inventory

| Family / active move IDs | Ordered active pose references | Objective visual gap | Next step |
|---|---|---|---|
| `jab` / `jab` | `jab0`, `jab1`, `jab0`, `idle0` | jab0.png and jab1.png show the older sleeveless navy/cyan outfit; idle0.png is the canonical return frame. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `strong` / `strong`, `launcher`, `escape_burst`, `lunge`, `halfmoon` | `str0`, `str0`, `str1`, `str1`, `str0`, `idle0` | str0.png and str1.png show the older sleeveless navy/cyan outfit; several moves reuse this animation. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `lkick` / `lkick` | `lk0`, `lk1`, `lk0`, `idle0` | lk0.png and lk1.png show the older sleeveless navy/cyan outfit; idle0.png is canonical. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `hkick` / `hkick`, `overhead` | `hk0`, `hk0`, `hk1`, `hk2`, `hk2`, `hk0`, `idle0` | hk0.png, hk1.png and hk2.png use the older outfit. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `cjab` / `cjab` | `cjab0`, `cjab1`, `cjab0`, `crouch` | cjab0.png and cjab1.png use the older outfit; cjab0 has a separately reviewed canonical-outfit candidate. | cjab0 visual candidate is approved pending runtime but unintegrated. cjab1 v7 remains unresolved (katana too short); v8 is rejected (full katana overflows fixed canvas by 43 px). Resolve a full-length-katana composition before promotion. |
| `clk` / `clk` | `crouch`, `clk`, `inline-rig-pose`, `crouch` | crouch.png, clk.png and crouch1.png use the older outfit. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `sweep` / `sweep`, `slide` | `sweep0`, `sweep1`, `sweep1`, `sweep0`, `crouch` | sweep0.png and sweep1.png use the older outfit; crouch.png is a shared old-design return. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `ajab` / `ajab` | `jumpTop`, `ajab`, `inline-rig-pose`, `jumpTop` | jumpTop.png, ajab.png and jumpDown.png use the older outfit. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `akick` / `akick`, `divekick`, `falcondive` | `akick0`, `akick`, `akick`, `jumpTop` | akick0.png, akick.png and jumpTop.png use the older outfit. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `lowKick` / engine-injected from `clk` | `crouch0`, `lowKick`, `crouch1`, `crouch` | The engine adds this move after JSON load. `lowKick.png` is canonical; surrounding crouch entry/return frames use the old outfit. | Preserve `lowKick.png`; correct only old-design crouch0/crouch1/crouch frames and review the complete injected sequence. |
| `airLowKick` / engine-injected from `akick` | `jumpTop`, `airLowKick`, `jumpDown` recovery | The engine adds this move after JSON load. `airLowKick.png` is canonical; adjacent air attack/recovery frames still use the old outfit. | Preserve `airLowKick.png`; correct only confirmed old-design frames and review the complete injected sequence. |
| `stringA` / `string_a` | `str0`, `str0`, `str1`, `jab0`, `jab1`, `jab0`, `idle0` | Reuses str0.png/str1.png and jab0.png/jab1.png; correction required. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `stringB` / `string_b` | `jab0`, `jab1`, `lk0`, `lk1`, `str0`, `str0`, `str1`, `str1`, `str0`, `idle0` | Reuses jab0.png/jab1.png, lk0.png/lk1.png and str0.png/str1.png; correction required. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `stringC` / `string_c` | `hk0`, `hk0`, `hk1`, `hk2`, `hk2`, `hk0`, `idle0` | Reuses hk0.png/hk1.png/hk2.png; correction required. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `drawThrow` / `drawThrow` | `cr1`, `cr2`, `str0`, `idle0` | cr1.png/cr2.png and str0.png use old design; idle0.png is canonical return. | Use approved Kaname identity refs and create distinct art per phase; replace only confirmed aliases and retain already canonical frames. |
| `frostArc` / `frostarc` | `cr0`, `inline-rig-pose`, `cr1`, `cr2`, `idle0` | cr0.png/cr1.png/cr2.png are old-design poses, mixed with canonical inline_3f649693d432dce5.png. | Replace only old-design poses; preserve canonical inline_3f649693d432dce5 in frostArc and inline_325beea3d661fe78/inline_f4ead780ee459475 in super. Do not copy special-only VFX. |
| `frostArcEx` / `ex_frostarc` | `cr0`, `inline-rig-pose`, `cr1`, `cr2`, `idle0` | Same cr0/cr1/cr2 old-design sources as frostArc, mixed with canonical inline_3f649693d432dce5.png. | Replace only old-design poses; preserve canonical inline_3f649693d432dce5 in frostArc and inline_325beea3d661fe78/inline_f4ead780ee459475 in super. Do not copy special-only VFX. |
| `hoarfrost` / `hoarfrost` | `mr0`, `mr1`, `mr1`, `mr2`, `jumpDown` | mr0.png/mr1.png/mr2.png and jumpDown.png show the older outfit; already correction_required in source report. | Make separate approved candidates for mr0/mr1/mr2 across hoarfrost, EX and trap; review jumpDown separately. |
| `hoarfrostEx` / `ex_hoarfrost` | `mr0`, `mr1`, `mr1`, `mr2`, `jumpDown` | Same old mr0.png/mr1.png/mr2.png/jumpDown.png sources as hoarfrost; already correction_required. | Make separate approved candidates for mr0/mr1/mr2 across hoarfrost, EX and trap; review jumpDown separately. |
| `blinkCut` / `blinkcut` | `ss0`, `ss1`, `ss1`, `ss2`, `idle0` | ss0.png/ss1.png/ss2.png use the older outfit; source report static_pass is contradicted by native comparison. | Review each alias as a separate action pose; correct only old-design sources, then compare complete ordered move at native/2x. |
| `iai` / `iai` | `cr0`, `cr0`, `cr1`, `cr2`, `cr2`, `idle0` | Reuses old-design cr0.png/cr1.png/cr2.png; source report static_pass is contradicted. | Correct each distinct cr/sp phase once; retain canonical inline aliases/effects/timing, then update and review all active users together. |
| `trap` / `trap` | `mr0`, `mr1`, `mr2`, `idle0` | mr0.png/mr1.png/mr2.png show the older outfit; source report static_pass is contradicted. | Make separate approved candidates for mr0/mr1/mr2 across hoarfrost, EX and trap; review jumpDown separately. |
| `thousand` / `thousand` | `cr0`, `cr1`, `cr2`, `cr1`, `cr2`, `cr1`, `cr2`, `cr1`, `cr2`, `cr2`, `idle0` | Reuses old-design cr0.png/cr1.png/cr2.png across the super sequence; source report static_pass is contradicted. | Correct each distinct cr/sp phase once; retain canonical inline aliases/effects/timing, then update and review all active users together. |
| `thousandUlt` / `ult_thousand` | `cr0`, `cr1`, `cr2`, `cr1`, `cr2`, `cr1`, `cr2`, `cr1`, `cr2`, `cr2`, `idle0` | Reuses old-design cr0.png/cr1.png/cr2.png across the ultimate; source report static_pass is contradicted. | Correct each distinct cr/sp phase once; retain canonical inline aliases/effects/timing, then update and review all active users together. |
| `super` / `winterfang` | `sp0`, `inline-rig-pose`, `sp1`, `inline-rig-pose`, `cr2`, `idle0` | sp0.png/sp1.png/cr2.png are old-design poses, interleaved with canonical inline_325beea3d661fe78.png and inline_f4ead780ee459475.png; source report static_pass misses the outfit alternation. | Replace only old-design poses; preserve canonical inline_3f649693d432dce5 in frostArc and inline_325beea3d661fe78/inline_f4ead780ee459475 in super. Do not copy special-only VFX. |
| `needleBeam` / `byakurai` | `inline-rig-pose`, `inline-rig-pose`, `inline-rig-pose`, `cr2`, `idle0` | inline_2cdd134a69aabfbf.png, inline_4a5b82897bece4b5.png and inline_282e2507e149aa22.png show old outfit with cyan blade/aura. | Review each alias as a separate action pose; correct only old-design sources, then compare complete ordered move at native/2x. |
| `crimsonCannon` / `shakkaho` | `inline-rig-pose`, `inline-rig-pose`, `inline-rig-pose`, `cr2`, `idle0` | inline_f4f80989ec6dd44f.png, inline_385ffe023b743a7d.png and inline_6024007266145d23.png show old outfit with cyan aura. | Review each alias as a separate action pose; correct only old-design sources, then compare complete ordered move at native/2x. |
| `flashStep` / `shunpo_behind` | `inline-rig-pose`, `inline-rig-pose`, `inline-rig-pose`, `ss2`, `idle0` | inline_4362387def07ae7a.png, inline_4642bd6b75a305a8.png and inline_d580a69b7cf04c8d.png show the older outfit. | Review each alias as a separate action pose; correct only old-design sources, then compare complete ordered move at native/2x. |
| `petalStorm` / `kageyoshi` | `sp0`, `inline-rig-pose`, `sp1`, `inline-rig-pose`, `cr2`, `idle0` | sp0.png/sp1.png/cr2.png and inline_b3540c4f422f879c.png/inline_7bce21d00d833441.png show the old design; source report static_pass is contradicted. | Replace only old-design poses; preserve canonical inline_3f649693d432dce5 in frostArc and inline_325beea3d661fe78/inline_f4ead780ee459475 in super. Do not copy special-only VFX. |


### Candidate status and coverage limit

- `cjab0.png`: the isolated candidate is approved as a visual pixel-art candidate pending runtime review; it is not integrated.
- `cjab1.png`: v7 passes 400×400, y=392, binary alpha, shared palette and ΔE≤0.13, but its katana became too short. V8 restores a comparable blade but exceeds the right canvas edge by 43 px at anchor (200,392), while the processed body is 197 px high versus 288 px in active `cjab1`. Both remain isolated and unapproved; see `candidates/kaname-cjab1-pilot/candidate-review-pipeline-v7.json` and `candidate-review-pipeline-v8.json`.
- `lowKick.png` and `airLowKick.png`: visually canonical files. `low_normals::complete` injects their moves/animations after JSON loading; roster integration tests cover this path but were not run during this art pass.
- Runtime alias resolution, animation continuity and in-game appearance remain unreviewed. No active sprite or character data changed in this inventory update.
