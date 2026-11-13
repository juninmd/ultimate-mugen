# Static inventory of attacks used by transformed forms — 2026-11-12

Scope: every `abilities.forms[].moves[]` entry for the 16 playable native fighters. This inventory is separate from the three `powerUp` transition frames. It resolves each form move to its declared animation and currently referenced body poses, then compares those poses with the fighter's canonical identity and the declared form palette. It does not claim runtime playback or collision correctness.

## Form move resolution

| Fighter / form | Move → animation | Resolved body poses | Classification |
|---|---|---|---|
| Akio awakened / ascended | `awk_starburst` → `starburst`; `asc_ult_cometstorm` → `stormSuper` | `sb0`, `sb1`, `sb2`, `jab0`; `nc0`, `nc1`, `sb1`, `sb2`, `cu1`, `jumpTop` | current pack identity; no separate outfit declared; `RIG`/effect slots open |
| Bruno awakened / ascended | `awk_quake` → `quake`; `asc_ult_titansuplex` → `titanSuplex` | `qs0`, `qs1`; `sl0`, `sl1`, `sl2`, `br0`, `br1` | current pack identity; no separate outfit declared; effect/rig slots open |
| Daichi awakened / ascended | `awk_ridgeblast` → `ridgeBlast`; `asc_ult_avalanche` → `avalanche` | `sb0`, `sb1`, `sb2`, `jab0`; `sb0`, `mh0`, `mh1`, `mh2` | current pack identity; palette/form declaration only; repeated frames are intentional sequence reuse |
| Garan awakened / ascended | `awk_hellorb` → `hellorb`; `asc_ult_onirush` → `oniRush` | `ho0`, `ho1`, `ho2`; `da0`, `da1`, `da2`, `bd0` | current Garan identity where resolved; no form outfit replacement; effect slots open |
| Haruki awakened / ascended | `awk_sigil` → `sigil`; `asc_ult_proof` → `proof` | `gale0`, `gale1`, `gale2`, `jab0`; `sup0`, `tk0`, `bar1`, `bar2`, `bar3`, `rise1`, `rise2`, `idle0` | current identity; awakened move reuses the unresolved gale family; ascended move uses current body poses |
| Jin awakened / ascended | `awk_chakram` → `chakram`; `asc_ult_shadowslash` → `shadowslash` | `kv0`, `kv1`, `jab0`, `idle0`; `tp0`, `tp2`, `sf1`, `sf2`, `sf3`, `sf4` | current identity/palette; no separate outfit declared; full visual approval remains open |
| Kaito awakened / ascended | `awk_gale` → `gale`; `asc_ult_triad` → `triad` | `gale0`, `gale1`, `gale2`, `jab0`; `sup0`, `tk0`, `bar1`, `bar2`, `bar3`, `rise1`, `rise2`, `idle0` | current identity; form palette only |
| Kaname awakened / ascended | `awk_frostarc` → `frostArc`; `asc_ult_thousand` → `thousand` | `cr0`, `cr1`, `cr2`, `idle0`; repeated `cr1`/`cr2` sequence | current identity; no outfit replacement; unresolved cjab family is separate |
| Lian awakened / ascended | `awk_lotus` → `lotus`; `asc_ult_storm` → `storm` | `lp0`, `lp1`, `jab0`, `idle0`; `ca0`, `hp1`, `hp2`, `hp3`, `ca1`, `ca2` | current identity; no outfit replacement; rig/effect slots open |
| Marlow awakened / ascended | `awk_riptide` → `riptide`; `asc_ult_kraken` → `kraken` | `rs0`, `rs1`, `rs2`; `ws0`, `ws1`, `ws2`, `al1`, `rs2` | current pirate identity; palette/form declaration only |
| Oboro forms | `corte_eclipse` → `espelho`; `chama_eclipse` → `amaterasu`; `julgamento_eclipse`/`supernova_primordial` → `kyoka` | `ss0`, `ss1`, `ss2`; `cr0`, `cr1`, `cr2`; `sp0`, `mr0`, `hk1`, `sp1` | declared Oboro form identity; no old-family cross-character source found; rig/effect slots open |
| Shion awakened / ascended | `awk_crescent` → `crescent`; `asc_ult_eclipse` → `eclipseCleave` | `cr0`, `cr1`, `cr2`, `idle0`; `mr0`, `mr1`, `mr2` | current identity; palette/form declaration only |
| Suzume forms | `danca_sombras`/`corte_fantasma` → `strong`; `lotus_julgamento` → `judgement` | `str0`, `str1`; `ks0`, `ks1`, `ks2` | intentional form move reuse; canonical Suzume identity must still be checked per pose; no separate outfit proof |
| Sylene awakened / ascended | `awk_lance` → `lance`; `asc_ult_crown` → `crown` | `fl0`, `fl1`, `jab0`; `gs0`, `gs1` | current mage identity; no outfit replacement; effect/rig slots open |
| Valdric awakened / ascended | `awk_royallance` → `royalLance`; `asc_ult_barrage` → `barrage` | `sb0`, `sb1`, `sb2`, `jab0`; `nc0`, `nc1`, `sb2` | current identity; royalLance sb1 is the reviewed candidate now active; runtime still open |
| Volta awakened / ascended | `awk_arcbolt` → `arcbolt`; `asc_ult_relay` → `relay` | `ab0`, `ab1`, `ab2`; `tc0`, `tc1`, `ab2`, `od0`, `od1` | current identity; palette/form declaration only; shared-palette QA for base jab is separate |

## Old-family versus legitimate form changes

No resolved form attack was found to use a different character identity. All body PNGs resolve within the current fighter pack and preserve the declared form's palette or effect treatment. The forms mostly use palette/form declarations (`replaceMoves: false`) rather than a separate outfit sprite family. A reused base animation is therefore not evidence of a transformation sprite bug by itself; it remains open until the pose is reviewed against the form palette and canonical identity.

The unresolved cases are the explicit `RIG` pose entries and procedural effect/projectile slots, plus any form move whose referenced base family is already correction-required in the full-family audit. Those must not be reported as visually approved merely because the form declaration resolves.

## Hash and evidence sources

For every resolved pose listed above, the full SHA-256 is authoritative in `native-size-page-manifest.json`; the compact pose list is derived from the current character JSON and animation definitions. `powerUp` hashes are intentionally excluded here and remain in `transformation-static-audit/powerup-source-review.json`. This report makes no runtime claim: transition-cycle and form-attack captures are still required for all 16 fighters.

Conclusion: form attack declarations resolve to current pack identities, but this is an inventory and static source audit, not approval. Any base family marked correction-required or unresolved remains open inside the transformed form as well.
