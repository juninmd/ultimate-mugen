# Native sprite review checkpoint

## Current continuation status — 2026-11-12

- This status supersedes the frozen-round runtime statements below; those rounds are retained as historical evidence.
- The current consolidated inventory still reports 1,703 family rows: 804 static pass, 741 correction-required, 158 unresolved. This is a pre-promotion baseline; hashes and visual dispositions must be refreshed after the correction set is frozen.
- Independent family reports cover 14/16 playable fighters (1,133 family rows); their pre-promotion dispositions were 558 static pass, 512 correction-required and 63 unresolved. The source manifests are stale after the current shared-frame corrections, so no adjusted family total is claimed here. Regenerate family dispositions, hashes and native-size contact/page manifests after the artwork is frozen. Kaname and Valdric are covered by separate reports outside that 14-character subset.
- Integrated visual corrections in this continuation: Garan `jumpTop`, `jumpDown`, `jumpUp`, `hop`, `sg_run_mid` top-edge clipping and an `airLowKick` top-edge clipping fix; Suzume five crouch/guard poses plus `guard`/`guardHit`/`airGuard`/`airGuardHit` and `hitCrouch`/`hitL1`/`hitL2`/`hitL3`; Haruki three-pose jab, three-pose `gale` sequence, high-reuse `str0`/`str1` and low-kick `lk0`/`lk1`; Volta three-pose jab; Valdric `jab0` and `sb0`/`sb1`/`sb2`; Kaname `crouch`, `guard`, `guardHit`, `mr0` and `mr2` (cyan trails retained). Backups, hashes and comparisons are retained under `integrated/` and per-family reports. The consolidated structural/shared-palette audit covers 40 promoted images across six fighters; Garan `sg_run_mid` has a separate structural report at `candidates/garan-sg-run-mid-edge-v1/promotion-structural-qa.json`. Runtime for these new frames remains pending.
- All 16 playable low kicks have prior runtime connection captures; stable transformed idle states also have runtime captures. Neither fact validates every attack family or the three-frame transformation transition. Form attacks are mapped separately; any base family still marked correction-required remains open in forms.
- Five NPC clone/source reviews identify `npc_colossus` as unresolved pending rendered-palette comparisons, `npc_phantom` as a large correction queue, and other family/FX gaps; story rendering alone is not pose approval.
- `git diff --check` passes. No test suite or browser smoke has been run during this correction pass; final tests, full gameplay/browser runtime and PR remain open until art is frozen and evidence regenerated.


Review ID: NATIVE-ROSTER-2026-11-12-A
Mode: improve; this checkpoint records a non-converged improvement pass
Epoch: geometry-normalized current worktree; base commit `881b8db6e20858209f3de4237111a28266a44bd1`
Artifact: 21 native packs in `assets/chars/sprites/`, the 21 UMF JSON sources, and `generate_missing_sprites/main.rs`
Scope: all 2,175 combat PNGs, 1,703 declared animation families, and 21 resolved power-up families. Excludes portraits, stages, imported M.U.G.E.N. assets, and runtime quality claims beyond captured states.
Historical artifact digest from the initial frozen checkpoint (not the current regenerated NPC state): SHA-256 `6c92745c437ae2b30846b33ebafcecdac8322e6227cbab2ca27c5fb3dd57402b` over 2,175 PNGs, 21 metadata files, and 21 native fighter JSON files (2,217 files; 119,355,521 bytes).
Protected surfaces: canonical character identity, pose readability, 400×400 transparent combat canvases, pack scale/floor anchors, attack behavior, and unrelated existing worktree edits.
Privacy/licence boundary: local repository assets only; no external sprite source or account data used.
Owners: host=root; clarification=root; artifact mutation=root; runtime=root; review=root (role-separated self-review, not independent); promotion execution=root, previously authorized by the user but not used in this checkpoint. No handoffs.
File state: existing worktree corrections are preserved. The 26 geometry metadata repairs are in the current candidate; exact pre-repair copies were not made before those edits. Git baseline and audit report remain available for tracked files. No asset mutation occurred after this epoch was frozen.
Runtime state: process-isolated captures; no persistent runtime mutation. Only Garan's transformed form has a current runtime screenshot in this evidence folder; the other playable transformations remain unverified in runtime.
Acceptance authority: user-provided `AGENTS.md` sprite standard; final appearance approval remains with the user.
Claim ceiling: technical normalization is clean; all-roster visual standardization is not established.
Bound: one review packet, maximum 90 minutes or four material-finding rounds. Stop/reopen: preserve open findings, start a new epoch for any sprite mutation, and do not open the PR until the requested corrections and required evidence are reviewable.

## Round 1 — current frozen epoch

- Reviewer: role-separated self-review; not independent.
- Evidence: regenerated 21 contact sheets; engine resolver inventory; per-file geometry audit; transformation contact sheet resolved from the engine's actual sources; Garan transformation runtime capture.
- Findings fixed: 16 out-of-range `axisY` values clamped; 10 missing frame geometry entries populated. Follow-up audit: zero canvas or metadata anomalies.
- Findings open: confirmed auxiliary identity mismatches in Bruno, Daichi, Garan, Kaito, Lian, Marlow, Sylene, Valdric, and Volta; 1,703 families still lack pose-level dispositions; 69 contact-sheet frames touch a top or side edge and need clipping review; 15 playable transformation families lack runtime review; the image-generation route remains quota-limited and local Grok is unauthenticated.
- False positive closed: Kaito's transformation uses exact `powerup0.png`–`powerup2.png`; resolver reports three unique images, confirmed in the corrected contact sheet.
- Rejected candidate: local rig render for Garan `hop` did not meet the authored pack's visual standard; original restored. The previous broad generation pilot's 44 unreviewed files were discarded.
- Verification: all 21 packs report zero uncovered resolver requirements; all 21 `powerUp` animations resolve three unique images; all 2,175 combat PNGs are 400×400; metadata audit has zero anomalies; `git diff --check` passes. Tests and full runtime tour are pending for the final validation stage.
- Result: not clean; convergence streak 0. No PR opened.

## Round 2 — same frozen artifact epoch

- Reviewer: role-separated self-review; not independent.
- Frozen scope confirmed: yes; no combat PNG, metadata, or fighter JSON changed since the recorded digest.
- New evidence: `transformation-family-review.png`, `edge-touch-review.png`, and `pixel-edge-palette-screen.json`.
- Findings: all 21 `powerUp` families have three unique resolver images; no obvious transformation identity switch at contact-sheet scale. Kaito filename false positive closed. Across all combat PNGs, maximum opaque palette size is 48 and partial-alpha pixel count is zero. Edge contacts are mostly bottom-baseline pixels; top/side cases remain open for native-size review.
- New material findings: none confirmed beyond the existing identity mismatches and coverage gaps.
- Artifact/runtime mutation: no; Garan runtime screenshot remains the only playable transformation runtime evidence.
- Verification: transformation resolver inventory plus current contact sheets; pixel/alpha screening across all 2,175 combat PNGs; geometry audit remains at zero anomalies.
- Result: not clean because pose-level identity review, clipping adjudication, and 15 transformation runtime reviews remain open. Clean streak remains 0; no PR opened.

## Round 3 — generator capability probe, same asset epoch

- Reviewer: role-separated self-review; not independent.
- Frozen asset scope confirmed: yes; no sprite or metadata file changed.
- Probe: dry-run with `--character=daichi --rebuild-list=daichi/cjab0.png` did not target `cjab0`; the frame is an explicit sprite override. It only proposed three unrelated missing inline poses.
- Result: the in-repo rig generator cannot replace this confirmed identity-mismatch family from the existing animation data. No candidate was produced or promoted. Image generation/authentication remains the available route for new pose art.
- Result: not clean; convergence streak remains 0.

## Round 4 — native-size Garan jump-family comparison, same frozen asset epoch

- Reviewer: role-separated self-review; not independent.
- Frozen asset scope confirmed: yes; no combat PNG or metadata changed.
- Comparison: canonical `04_special.png` against `hop.png`, `crouch.png`, `jumpDown.png`, `jumpStart.png`, `jumpTop.png`, `jumpUp.png`, and `land.png` at native 400×400.
- Findings: `hop`, `jumpDown`, `jumpTop`, and `jumpUp` are confirmed identity mismatches. `crouch`, `jumpStart`, and `land` match the current Garan identity at native size. The exact affected families and current file hashes are in `garan-jump-backlog.json`; one-pose prompts are prepared in `garan-jump-prompts.md`.
- Result: new material evidence adds three mismatched jump poses beyond the previously tracked `hop`. Review bound reached at four rounds; checkpoint retained and a new bounded correction packet is warranted for these exact frames.
- Result: not clean; convergence streak 0; no PR opened.

# Native sprite correction packet B

Review ID: NATIVE-GARAN-JUMP-2026-11-12-B
Mode: improve
Epoch: same frozen asset digest as packet A; evidence epoch advanced by native-size Garan pose comparison and exact file backlog.
Scope: only Garan `hop.png`, `jumpDown.png`, `jumpTop.png`, and `jumpUp.png`; their affected animation-family references are enumerated in `garan-jump-backlog.json`.
Explicit exclusions: other Garan poses, other characters, portraits, gameplay rules, and transformation families.
Acceptance: preserve Garan's canonical identity from `04_special.png`; keep each target's pose direction/readability; hard-edged pixel clusters; transparent 400×400 result; pack geometry axis (200,392), refHeight 320; no unwanted effects.
Owners: host=root; clarification=root; artifact mutation=root; runtime=root; review=root (self-review, not independent); promotion=root, not used here. No handoffs.
File layer: retain current target hashes listed in backlog and write additive candidates before promotion. Candidate-to-target mapping stays one-to-one. Runtime layer: process-isolated fighter tour after candidate approval, with outputs kept in a separate evidence folder.
Bound: up to four candidate/review rounds or 90 minutes. Stop if generation remains unavailable; keep original files and preserve the exact blocker.
Status: not yet converged; no target sprite changed. A generation attempt at 15:00 UTC returned HTTP 429 (`usage_limit_reached`, reset timestamp 2026-11-12T19:37:08Z). Keep candidates additive and retry one `hop` generation only after that limit resets; inspect and normalize it before considering the other three poses.

# Supplemental full-roster triage — 2026-11-12

All 21 refreshed contact sheets were screened against the canonical poses visible in each pack. The findings and per-pack dispositions are recorded in `visual-findings.md`. This was a non-mutating triage: contact-sheet screening alone does not close per-frame review or grant runtime approval. Native-size comparisons then confirmed three Oboro jump-frame mismatches, with target and anchor hashes and affected animations in `oboro-jump-backlog.json`; one-pose prompts are in `oboro-jump-prompts.md`. The initial Shion concern was not confirmed. No sprite, metadata, or fighter JSON changed in this pass. The full checklist remains pending, and the identity mismatches previously confirmed for Bruno, Daichi, Garan, Kaito, Lian, Marlow, Sylene, Valdric, and Volta remain unresolved except for the corrections already listed in `visual-findings.md`.

Follow-up native-size comparisons covered 104 distinct PNGs across eleven packs and tagged 285 resolved-frame occurrences in `family-review-checklist.json`. The tags distinguish confirmed visual inconsistency, retained identity needing style review, and sampled identity consistency. They do not close any containing family because its other resolved frames and runtime sequence remain unreviewed. The unified backlog contains 200 files across ten packs: 82 confirmed inconsistencies, two identity-consistent samples, 14 identity-retained/style-pending samples, and 102 not yet reviewed at native size. Daichi's flip and get-up sequences contain a confirmed outfit switch between adjacent frames (`flip0`/`flip1-3`, `getUp1`/`getUp2-3`). All 28 Volta candidates and 28 Bruno candidates have been individually compared at native size and await replacement. The Oboro and Garan jump mismatches have hash-pinned entries.

One-pose correction prompt maps are prepared for the 28 Volta, 28 Bruno and 19 Daichi confirmed candidate files (`volta-correction-prompts.md`, `bruno-correction-prompts.md`, `daichi-correction-prompts.md`). Each target is pinned to its current hash and affected animation list. Generation remains single-pose with a pilot review per pack; no prompt plan is treated as a generated or approved asset.


## Native-size roster review update — 2026-11-12

Five assigned audit agents were used as requested. Four candidate-set reviews covered 107 frames across Lian, Marlow, Sylene and Valdric; their results remain 102 identity-consistent and five style-review-pending. Additional focused checks found 11 Jin and nine Haruki native-size style-family mismatches; Haruki has 46 more contact-sheet candidates not individually confirmed. Akio has no new confirmed defect from 20 native-size samples, with 77 of its 97 resolved PNGs not individually reviewed. The current 267-frame backlog across 14 packs contains 102 confirmed inconsistencies, 104 identity-consistent frames, 15 style-review-pending frames and 46 not yet inspected at native size. The family checklist records 610 tagged frame occurrences across 227 distinct PNGs; all 1,703 family-level `visualReview` states remain pending.

The first attack audit summary and Akio runtime capture were inadequate: the former used a stale inventory, and the latter was obscured by transition effects and CPU attacks. Both have since been replaced with current evidence, documented below.


The current native attack audit inventory was regenerated after the earlier stale-report discrepancy. Across 681 authored attacks from all 21 native characters (526 playable and 155 story NPC), all resolve at least three distinct body images in the fresh inventory. Both dynamic lowKick and airLowKick resolve four images per character; unit coverage now includes all 21. The cargo test remains pending. Details: `attack-keyframe-gaps.json`.


## Transformation runtime close-out for playable roster — 2026-11-12

Generated native-size comparisons for all 21 `powerUp` families against their canonical identity anchors. Captured post-transformation runtime idle states for all 16 playable fighters in isolated desktop processes; the stable shots show the expected character and form palette without the former CPU attack occlusion. Contact sheet pages: `runtime-transform-audit/awakened-runtime-page-1.png` through `runtime-transform-audit/awakened-runtime-page-4.png`; source screenshots and logs are in per-fighter subdirectories. This closes transformed-idle-state runtime coverage only. The three-frame transition animation itself remains to be visually inspected in runtime, and all 1,703 family-level checklist rows remain pending until their full sequences are reviewed.

The fresh `native_sprite_audit` inventory shows 681 authored attack entries across all 21 native characters and zero below three distinct rendered images. Both lowKick and airLowKick resolve four body images in every pack. Regenerating the five story NPC JSON files from their current fighter bases raised their authored sequence count from 20–21 to 31 each and fixed three stale two-image normal sequences per NPC. The all-character integration/unit test updates remain unrun until final validation.


The low-kick unit coverage was tightened so the all-roster loop now checks both generated `lowKick` and `airLowKick` animations resolve at least three distinct body images; the existing exact frame-sequence assertion remains. This test change has not yet been run; final test suites stay deferred until the correction pass is complete.

## Current worktree audit epoch C — 2026-11-12

Mode: improve. Artifact: current fix/native-roster-low-kicks worktree at base 881b8db6; current review digest SHA-256 7eb10b42eec44f2d204a497d3cd9bd540f6c948c849fc301499db53d270eca62 over 2,171 combat PNGs, 21 sprite metadata files and 21 native fighter JSON files (2,213 files; 119,309,370 bytes). The digest includes each relative path and file bytes. Portraits, sprite index and imported M.U.G.E.N. assets are excluded. Four resolver-unreferenced old-design Akio PNGs and their stale per-frame metadata entries were removed after hash-verifying copies into orphan-cleanup/akio-before/.

Review owners: root owns scope, artifact mutation and runtime; five separate read-only audit agents covered disjoint packs; final user appearance approval remains with the user. This pass used process-free static inspection; runtime mutation is N/A. Acceptance authority is the user-provided native sprite standard in `AGENTS.md`.

### Coverage and findings

- Structural pixel payload: 2,171/2,171 PNGs are 400×400 RGBA with at most 47 opaque colors and no partial-alpha pixels; automated audit reports no payload issues.
- Native visual pages: 2,171/2,171 current combat PNGs were placed in 95 pages with filename labels and unscaled 400×400 tiles; five reviewers inspected their assigned page groups. Pack counts sum exactly to 2,171. The first attempted Akio-group count used a different checkout and was rejected; the explicit worktree recount and page review replaced it. The refreshed manifest and pages exclude four subsequently removed Akio orphans.
- Confirmed wrong-family/identity frames: Suzume `cjab0.png` and `cjab1.png`. Confirmed outfit-family switches: 36 animated Akio auxiliary sprites and 70 animated Kaname auxiliary sprites; both conflict with palette/effect-only forms.
- The 374-entry queue now pins current hashes and resolved animations for confirmed mismatches and style candidates across 14 packs. Haruki/Jin initial leads were adjudicated; Haruki has 29 style candidates, while Jin has no exact defect confirmed. Runtime families and effect-only questions remain open.
- Kaito's suspected alternate scarf was not present in the reviewed pages; all observed poses retained the yellow headband and declared form palettes do not change outfit. Kaname's dark blue/cyan armored auxiliary outfit conflicts with its canonical white haori and palette-only declared forms; 70 exact files are hash-pinned and mapped to resolved animations. Large ability/movement FX in Akio, Bruno, Daichi and Garan need per-state effect adjudication.
- The 282-entry backlog currently has 87 confirmed inconsistencies, 139 identity-consistent files, 56 style-review-pending files and no remaining native-size candidates. The separate correction queue has 374 files across Akio, Bruno, Daichi, Garan, Haruki, Kaname, Lian, Marlow, Oboro, Shion, Suzume, Sylene, Valdric and Volta (221 confirmed style/family mismatches, 36 confirmed Akio outfit-family mismatches, 70 confirmed Kaname outfit-family mismatches, 45 style candidates, and two wrong-family Suzume frames). Kaname entries are hash-pinned and mapped to resolved animations. The queue excludes effect-only questions. Haruki’s 29 former unconfirmed candidates are now native-reviewed style candidates. All 1,703 family checklist rows remain pending.
- Transform idle screenshots cover all 16 playable fighters; the transition cycles themselves remain unreviewed in runtime. NPC static review is clean across all 485 images; NPC runtime still needs coverage.
- Updated attack/low-kick tests, whole-roster runtime, browser smoke and final test suites have not been run; sprite correction is incomplete, so this is not the final validation epoch.

Result: not converged; clean streak 0. No PR opened. Next action is to generate and review replacement candidates against each character's canonical pose, re-render/review complete affected sequences, then run gameplay/runtime/browser and test gates once the art set is frozen.

The current `visual-family-correction-queue.json` pins 374 exact source files with per-file SHA-256 and resolved animation usage, including 70 Kaname outfit-family mismatches. It does not include effect-only ambiguity. Native review pages are indexed by `native-size-page-manifest.json`; regenerated page images were transient under the task temp directory and are not part of the repo. The source-file checklist remains pending until each correction candidate is generated, pixel-reviewed at native/enlarged scale, and its full affected animation is reviewed in runtime.


## Current worktree audit epoch D — 2026-11-12

Worktree: fix/native-roster-low-kicks at base 881b8db6. Review digest SHA-256 `a872e06fae2746bdf0209ec70743e7fcc8d22be1195e2e3227223d8acf95ba7b` over 2195 files (118,574,992 bytes): 2,153 current combat PNGs, 21 metadata files and 21 fighter JSONs. The digest includes each relative path and file bytes. Portraits, sprite index and imported M.U.G.E.N. assets are excluded.

Confirmed 18 additional resolver-unreferenced PNGs from Haruki, Jin, Lian, Oboro, Shion, Suzume and Sylene were removed after native-size comparisons with each pack's canonical `04_special.png`. Each source is archived under `orphan-cleanup/foreign-identity-before/` with SHA-256, and its stale frame geometry entry was removed from `meta.json`. Ambiguous candidates and all rig pose definitions were preserved. Earlier Akio and Garan orphan cleanup remains hash-backed.

The refreshed page manifest contains all 2,153 remaining combat PNGs across 21 packs in 94 labeled pages. The payload audit is PASS: all are 400×400 RGBA, no partial-alpha pixels, maximum 47 opaque colors, no canvas-size anomalies. The 379-entry correction queue was revalidated: all paths and hashes are current, with zero integrity errors. Lian `crouch0.png` is integrated from the reviewed candidate; static inspection is recorded, while complete affected-family runtime validation remains pending.

`native_sprite_audit --require-complete` passed all 21 characters with zero uncovered animation requirements. This confirms structural frame coverage only. The visual-family checklist still has 1,703 rows pending full family-level review; the remaining 378 correction candidates have no integrated replacement recorded, transformed idle coverage does not close transformation transitions, and NPC runtime coverage remains open. Full gameplay tests, browser smoke and the PR are deferred until art corrections and runtime review are complete.

Result: active; not converged; no PR opened. Next: generate one pose at a time from pack anchors, review complete affected sequences, validate transformations and NPC runtime, then run final test/browser gates.


### Native-size candidate adjudication update — 2026-11-12

A paired native-size pass resolved all 45 former style-review candidates. Haruki's 29 files are confirmed old-outfit frames: the same brown-haired identity remains, but plain purple shirts and dark trousers replace the approved long gold-trim coat, with no declared costume-changing form. Suzume's 16 files are confirmed wrong-identity frames: masked dark-purple ninja poses conflict with the canonical female green-and-cream fighter. Suzume's palette forms retain her design; `invocacao_oboro` explicitly targets the separate Oboro character and does not justify these base animation sprites. Five paired 400x400 comparison sheets and exact source hashes are in `family-candidate-adjudication/`.

Queue adjudication counts are now 162 confirmed style-family mismatches, 135 outfit-family mismatches, 63 visual-family mismatches, 18 wrong-identity frames, and one integrated Lian crouch candidate awaiting complete family runtime review: 379 entries total, all hash-current. Five Bruno candidates were rejected for identity drift and restored to their exact HEAD bytes. There are no unresolved native-size candidate statuses. All 1,703 full-family rows remain pending because their complete animation sequences and runtime behavior have not yet been approved.

### Five-agent round 2 — correction queue re-adjudication

Five read-only reviewers covered all 379 assigned queue entries across 14 packs. Individual 400×400 comparisons adjudicated all 36 Akio and 33 Bruno entries. The queue now records 15 Akio files as consistent/no correction, 21 Akio hair/face/rendering mismatches, and 33 Bruno finish/equipment mismatches; all hashes remain current. Other pack reviews confirmed material visual-family problems but left per-target or form-specific questions open as described in `visual-review-agents-round2.md`. The Suzume identity description in the older queue was explicitly contradicted and must not guide sprite generation. This round produced no replacement art: the local Grok CLI requires sign-in and host image generation is quota-limited until 2026-11-12 19:37 UTC. Runtime family validation remains open; the 1,703 family checklist rows are still pending.

### Full-roster family static audit — 2026-11-12

Five reviewers covered all 1,703 checklist rows and 6,117 ordered frame references across the 21 packs. A role-separated cross-review revised the 44 unresolved Kaname/Jin/Garan/Spitfire rows: 27 static pass, one additional correction (`Kaname/getUp`), and 16 remain unresolved. Consolidated row-level results and exact current source frame paths/hashes are in `full-family-static-review-current.json`; per-agent evidence is in `full-family-static-agent-1.json` through `full-family-static-agent-5.json` and `full-family-static-agent-2-crossreview-5.json`.

Disposition: 804 static pass, 741 correction required, 158 unresolved. These are family rows, not counts of files that must all be replaced. Path/order comparison against the current checklist has 1,703/1,703 exact matches; all 2,153 source PNG hashes match `native-size-page-manifest.json`. The checklist now records each static disposition and links to the consolidated report, while all native-resolution pose and runtime review fields remain pending. The 741 families reference an 884-PNG candidate source pool, deduplicated in `native-family-correction-backlog-current.json`; each target still needs individual review before replacement. Corrections newly uncovered beyond the earlier file queue include Jin, Kaito, Oboro, Valdric, Shion, and the Phantom `spriteSet=jin` source mismatch; NPC clone packs remain unresolved where rendered recolors were not verified. Static contact review does not approve every pixel cluster at native magnification, and no per-family runtime sequence was exercised. All runtime gates, 158 unresolved families, 741 correction findings, generation candidates, final tests and PR remain open.

### Full combat PNG structural pass — 2026-11-12

All 2,153 PNGs under `assets/chars/sprites/` excluding portraits were decoded and checked against the 400x400 combat canvas. No dimension mismatch, non-binary alpha pixel, or fully transparent frame was found. The per-file SHA-256 and alpha metrics are recorded in `native-pixel-structure-audit-current.json`, linked from the consolidated family review. This structural pass does not clear the 741 family correction findings or 158 unresolved family rows, and it does not substitute for native-scale visual or runtime review.

### Story-mode NPC runtime capture — 2026-11-12

Ran the compiled game through the full five-stage story tour at 1280x720. All fights were won and 78 screenshots were captured. Inspected the rendered NPC lineup and first wave; they display in the live renderer. This proves story integration and rendering only; it does not approve every NPC pose or sprite family, recolor, or animation. The screenshot inventory, hashes, exact claim ceiling, and inspected images are recorded in `storymode-npc-runtime/storymode-npc-runtime-audit-current.json`.

### Native-size adjudication correction — Haruki crouch0 — 2026-11-12

Re-reviewed the actual 400x400 source beside `04_special.png`, including a 2x nearest-neighbor evidence image. `crouch0.png` has the canonical long purple coat, gold embroidery/trim, cream lining, white gloves and dark trousers; the prior “old plain top” finding was incorrect. Reclassified this individual frame as consistent/no replacement, with pinned source and evidence hashes. Other Haruki frames remain independently queued; family-level corrections remain until every frame is reviewed and the full animation is validated.

### Suzume anchor description correction — 2026-11-12

The prior 16-frame adjudication used an incorrect green-and-cream description for `04_special.png`. Re-reviewed its 400x400 comparison pages against the actual purple-haired ninja with red bow, face mask and violet outfit. The 16 base frames remain material appearance mismatches, now described by the concrete hood, hair, silhouette, equipment and clothing differences; hashes and frame statuses are unchanged. They still require pose-preserving replacements and runtime family validation.

### Full-roster low-kick runtime capture — 2026-11-12

The current release binary rendered all 16 `Low Kick` and all 16 `Air Low Kick` states. Every ground low kick hit at a 36-unit showcase gap; eight missed at the default 52-unit gap, consistent with their short range. All 16 air low kicks hit at both distances. A first-hit-tick capture with `UME_BOXES=1` and a visible plains stage showed every airborne sprite and its collision overlay; this resolved a later-frame capture ambiguity seen in an earlier screenshot. Damage and screenshot hashes are in `low-kick-runtime-audit-current.json`. This is one-frame-per-move evidence, not full animation or roster sprite approval.

### Round 3 parallel roster review and Suzume finding correction — 2026-11-12

Five read-only agents independently rechecked the current family report across all 21 packs. Their per-pack counts and specific findings are in `visual-review-agents-round3.md`. The report still has 804 static pass, 741 correction-required and 158 unresolved family rows; every row remains pending native-resolution approval and runtime review. Cross-review confirmed 1,534 distinct referenced PNG paths exist with current hashes. This confirms audit coverage and source integrity, not visual approval.

The consolidated Suzume finding text had described its canonical `04_special.png` identity as unmasked, contradicting the anchor and `family-candidate-adjudication`. Corrected the 60 Suzume correction-family descriptions to specify the actual masked purple-haired kunoichi and the hooded/dark-armored mismatch. The corrected report SHA-256 is `ae861f983ccd298460ff6ba2a30b0abbb7bc96f88b138f70692d90102c382648`.

Generated two isolated Suzume crouch candidates with the Codex host image route. Both were preserved outside the asset pack. The second candidate was checked at native source size and failed `pixel_reduce.py --strict`: 1254x1254, 65,323 RGBA colors, 381,961 partial-alpha pixels, no clean pixel grid (score 0.012; minimum 0.5). It is rejected and not integrated. See `candidates/suzume-crouch-pilot/candidate-review-20261112.json`. No replacement sprite was integrated in this round.

### Five parallel art pilots — 2026-11-12

Five workers generated eight isolated Codex-host candidates for six targets (Suzume crouch, Haruki jab0, Kaname cjab0, Garan jumpDown, Valdric turn, Volta jab0). Every target's source SHA-256 was rechecked against the pilot manifest and remains unchanged. All eight outputs were rejected: seven failed strict logical pixel-grid QA (score below 0.5 and/or uniformity below 0.9), while the first Suzume output failed pre-screen dimensions and alpha inspection. Both Garan outputs also changed the target fall pose into a near-vertical guard; `airLowKick.png` was reviewed and excluded because it already matches Garan. Details and per-pilot evidence are in `visual-pilots-round1.md` and each `candidates/*-pilot/candidate-review*.json`. None was integrated. This confirms the current image route cannot produce acceptable replacements for these targets; the 741 correction-required family rows remain open.

### Pixel-finish pipeline validation — Haruki jab0 — 2026-11-12

A raw Haruki `jab0.png` generation was normalized with the native sprite pipeline and a balanced palette learned from the canonical `04_special.png` plus that registered frame. The output passes strict geometry QC and strict palette apply at max ΔE 0.1, measured 0.097131: 400×400, 43 RGBA colors including transparency, binary alpha, zero off-palette output pixels, zero partial-alpha output pixels, and the target `[200,392]` floor anchor. Native and 2× nearest review preserves the approved long purple/gold coat and setup pose. It is a viable isolated candidate, not integrated: `jab1` remains mismatched and the family still needs one shared palette and runtime review. Detailed metadata and screenshots are in `candidates/haruki-jab0-pilot/pixel-finish-review-20261112.json`.

### Shion idle family native review — 2026-11-12

Reopened the exact 400×400 Shion `01_idle`, `07_idle1`, and `08_idle2` sprites beside `04_special`, then reviewed a 2× nearest-neighbor comparison. The idle images retain the canonical long white hair, black robe, red trim, gray trousers, and broad sword; visible differences follow the idle versus attack pose. SHA-256 for `01_idle.png` still matches the family report. Kept the family as static pass and recorded the contact comparison at `shion-idle-native-review-2x.jpg`; no sprite changed.


### Suzume crouch-family correction — 2026-11-12

Replaced `crouch0.png`, `crouch.png`, and `crouch1.png` from isolated, pose-distinct candidates using the approved `04_special.png` identity anchor. Preserved the original PNGs and before/after SHA-256 values in `integrated/suzume-crouch-before/`. All three candidates were reapplied with the same 48-color palette, binary alpha, 400×400 canvas, axis (200,392), `refHeight` 320, and ground line y=392; strict palette ΔE values are 0.078079, 0.122040 and 0.104921 against a 0.13 cap. Rebuilt the labeled contact sheet from the integrated files at native and 2× nearest sizes. An independent reviewer checked the integrated crouch poses and guard-hit against the canonical anchor, active hashes, shared palette, scale and silhouettes; no static correction was found. This closes static identity and pose review for these three images only; runtime sequence validation remains pending, `crouchGuard`/`crouchGuardHit` remain open, and the consolidated family report still contains its pre-promotion hashes pending regeneration. No test or PR was run/opened in this pass.


### Suzume crouch-guard correction — 2026-11-12

Integrated `crouchGuard.png` and `crouchGuardHit.png` from isolated, pose-distinct candidates using the same 48-entry palette as the corrected crouch family. Original PNGs and before/after hashes are in `integrated/suzume-crouchguard-before/`. Both remain 400×400 RGBA with binary alpha, axis (200,392), refHeight 320 and floor y=392; strict ΔE values are 0.082346 and 0.070542 under 0.13. Labeled native/2× nearest sheets were rebuilt from active files. An independent reviewer also verified all five integrated crouch/guard assets against their manifests and shared palette at native/2× sizes, with no static correction found. This closes static identity/pose review for these two frames; runtime review remains pending, and `hitCrouch` is open.


### Volta jab three-pose correction — 2026-11-12

Integrated the Volta jab guard, impact, and recovery candidates as `jab0.png`, `jab1.png`, and new `jab2.png`; preserved original jab0/jab1 and JSON/meta snapshots in `integrated/volta-jab-before/`. The animation now uses jab0 (3 ticks) → jab1 (3) → jab2 (4) → idle0 (2), with the hit still on jab1 and move hit element 2; total duration remains 12 ticks. Added `meta.frames.jab2` at axis (200,392), refHeight 320. The three frames are 400×400, binary alpha, and use the shared 48-color palette; strict max ΔE values are 0.082817, 0.085590, and 0.093804 under 0.13. Labeled native and 2× nearest contacts are in `integrated/volta-jab-before/`. Visual sequence review is static; runtime and collision alignment remain pending, and other Volta families remain open.

### Haruki standing-jab correction — 2026-11-12

Integrated the independently reviewed shared-palette jab candidates as `jab0.png`, `jab1.png`, and new `jab2.png`; preserved the previous `jab0.png`, `jab1.png`, `meta.json`, and `haruki.json` under `integrated/haruki-jab-before/`. Native and 2× nearest contact sheets show guard/setup → full extension/hit → retracted recovery; identity details match `04_special.png`, and the active canonical `idle0.png` remains the terminus. All three frames share palette SHA-256 `709b253c…bebfaa2c`, strict max ΔE 0.110928/0.13, binary alpha, 400×400 canvas, axis (200,392), refHeight 320 and floor y=392. The jab animation keeps its 10-tick duration and hit on jab1: 2+3+3+2 ticks. Static review is complete; runtime timing/collision validation remains pending. See `haruki-jab-integration-20261112.json`.


### Volta jab shared-palette reapply — 2026-11-12

Resolved the independent review finding that active `jab0.png` used a different palette from `jab1`/`jab2`. Backed up the prior jab0 PNG and prior native/2× contacts, then reapplied the exact seed-13 48-entry shared palette. Strict apply passed at max ΔE 0.056425/0.13 with zero off-palette output pixels and zero partial alpha. All three active jab frames are now exact RGB subsets of the shared palette; active jab0 SHA is `d51d88b3…f65a8c`. Rebuilt native and 2× nearest contacts with the `04_special` anchor. Pose identity, progression, floor/scale and animation timing remain statically reviewed; jab1 hit stays on element 2 and duration stays 12 ticks. Minor brightness distribution difference remains between jab0 and jab1/jab2. Runtime and collision alignment remain pending. Provenance/backups: `integrated/volta-jab-palette-reapply/`; review: `volta-jab-independent-review-20261112.md` and `.json`.

### Independent roster audits — Akio, Bruno and Jin — 2026-11-12

Character-level family reviews complement the 1,703-row roster audit. Akio: 81 animation families reviewed; 58 static passes, 19 require visual correction or pose/continuity review, and 4 effects remain unresolved. Canonical `04_special.png` SHA-256 `8667e3340ff9bc0b5df7f1202b76e2608e287a1a8cecc4fdc7080422e67014ae`. See `akio-independent-family-review-20261112.md`.

Bruno: 79 families reviewed; 41 static passes, 36 require visual correction and 2 procedural effects remain inconclusive. Canonical `04_special.png` SHA-256 `ccca15b3a1b8529db9538dc0ccf1fa23d50ac87e74c0f95fabbe840a537486e9`. See `bruno-full-family-independent-review-20261112.md`.

Jin: 80 families reviewed; 27 static passes, 49 require correction and 4 effects are unresolved. Canonical `04_special.png` SHA-256 `e2c04824cdf4b009d245c5b77a58e775a7ad62f0815a731f18279166fcd0afc8`. The report identifies attacks, defense, locomotion and special families with old/inconsistent pose reuse; this family classification does not by itself establish wrong identity. Source reports used: `full-family-static-review-current.json`, `confirmed-family-frame-backlog.json` and `attack-keyframe-gaps.json`.

All three reviews are static. They do not validate runtime, hitboxes, transformations or animation timing; no assets were modified by the reviewers.

### Independent roster audits — Kaito and Lian — 2026-11-12

Kaito: 81 families reviewed against `04_special.png` SHA-256 `4bc460e9cb1dfd6b843dabe236a02b1907eadff51fc34dfc3b226e3d04069369`; 40 static passes, 38 correction-required and 3 unresolved effect families (`galeOrb`, `galeHit`, `dragonSpirit`). Main findings are old pose reuse and continuity issues across attacks, guard, hit, fall and recovery. The review does not infer wrong fighter identity from these mismatches. Runtime, hitboxes and forms remain unvalidated.

Lian: 79 families reviewed against `04_special.png` SHA-256 `248d8a4e6b75c14ea3528c9482301246d34efb05afcab5a8abc1f66f07c6d452`; 36 static passes, 41 correction-required and 2 unresolved procedural effect families (`petalFx`, `petalHit`). Crouch, guard, jump, attack, defeat and low-kick families retain appearance or continuity mismatches. The integrated `crouch0.png` does not close its full animation family. Runtime remains unvalidated.

### Independent Oboro family audit — 2026-11-12

Oboro: 87 families reviewed against `04_special.png` SHA-256 `fc7c788210afff2bb44fa620105f7298a8881f5e237d6fa3a03ab4a0e9bcb68`; 17 static passes, 60 correction-required and 10 unresolved procedural/inline effect families (`crowFx`, `crowHit`, `blackFlame`, `blackFlameHit`, `moonBeam`, `moonHit`, `spectral`, `spectralWave`, `spectralHit`, `kyokaFlame`). The body sprites generally retain Oboro's canonical identity; the extensive queue mainly reflects old pose reuse and continuity, not proven identity substitution. Static review only; runtime, hitboxes and transformation playback remain open.

### Independent Shion family audit — 2026-11-12

Shion: 82 families reviewed against `04_special.png` SHA-256 `86f4a9d31d3d1b3b2b7ab225171159a46598ea2061c5d9219eee690fc367cdd7`; 19 static passes, 59 correction-required and 4 unresolved procedural effects (`crescentFx`, `crescentHit`, `requiemFx`, `requiemHit`). The main issues are inconsistent finish/equipment across attacks, special moves, defense, strings and low kicks; identity remains broadly recognizable, so mismatched style is not treated as proof of another fighter. Runtime, collision and transformation checks remain open.

### Kaname low-kick linkage correction — 2026-11-12

The Kaname report initially described the low-kick files as unlinked because `kaname.json` does not serialize those moves. Source review of `crates/engine/src/native/low_normals.rs::complete` shows the engine injects them after JSON loading from `clk` and `akick`; their sprite families therefore are active. The family report now reflects the dynamic linkage and its ordered frames. The full visual family still needs correction because adjacent crouch/air poses retain the older outfit. Roster tests cover injection and distinct body images but were not run during this art pass.

### Independent Sylene family audit — 2026-11-12

Sylene: 80 families reviewed against `04_special.png` SHA-256 `bbb8675275a0ce31293f82b4a7a9898519c1b020e5a11250842309c8c4179c58`; 73 static passes, 3 pose/finish corrections (`airGuardStart`, `airGuard`, `airGuardHit`) and 4 unresolved effects (`lanceFx`, `iceHit`, `spireFx`, `winterBeam`). No identity mismatch was found. The correction must retain the canonical silver/cyan hair and light-blue robe while restoring volume and pixel-cluster readability. Runtime remains open.

### Independent Marlow family audit — 2026-11-12

Marlow: 80 families reviewed against `04_special.png` SHA-256 `8f90d80b40e0fd560da681bd1c345d13d13109d1b1ee120782fc25abbc0fd1e3`; 53 static passes, 24 require pose/finish correction and 3 procedural effect families remain unresolved. No identity error was found. Corrections cluster around crouch, guard, low-hit, crouch attacks, recovery, defeats and the ground low kick; the air low kick passed static review. Runtime remains unvalidated.

### Valdric royalLance correction — 2026-11-12

Integrated the three reviewed canonical candidates as `sb0.png`, `sb1.png`, and `sb2.png`; original images and metadata are preserved in `integrated/valdric-royallance-before/`. The phase sequence is charge (two declared red-pink orbs) → extension/hit (one orb) → empty-hand recovery. Native and nearest-2× contacts confirm the canonical Valdric armor, hair, sash and pose progression. Shared palette and registration gates pass: 400×400, binary alpha, axis (200,392), 48 colors, maximum ΔE 0.104288/0.13. `sb1` has 10px of right clearance but is not clipped. Animation JSON and timing are unchanged. Runtime and the other shared aliases (`lanceWall`, `royalLanceEx`, `super`, `barrage`, `barrageUlt`) remain open for review; report: `valdric-royallance-integration-20261112.json`.

### Independent Daichi family audit — 2026-11-12

Daichi: 81 families reviewed against `04_special.png` SHA-256 `2fba54b74d6b308c525e9d9663ffc1c569708ba23cf74738ea910168d1e8596d`; 56 static passes, 21 correction-required and 4 unresolved effects (`ridgeOrb`, `ridgeHit`, `sunbreakBeam`, `sunbreakHit`). Canonical identity is preserved; the open families show pose/finish and missing energy-cluster consistency. No runtime, collision or transformation approval.

### Independent Garan family audit — 2026-11-12

Garan: 80 families reviewed against `04_special.png` SHA-256 `1079c6081d3c350989b7389790dc6a7a6ae22ea9c709d3479aad9c4d46cf8b0a`; 56 static passes, 12 contain wrong-identity sprites and 12 remain unresolved due to floor/edge issues or missing body sources for effects. The wrong identity is the small dark horned fighter, not the canonical red oni with white mane, yellow horns and purple hakama. This independent family review does not close the broader visual correction work or runtime review.

### Independent Haruki family audit — 2026-11-12

Haruki: 79 families reviewed against `04_special.png` SHA-256 `cbd71acba79973deed0d36bf789b015a96d5832482ee0f3af9c54900821a24a9`; 24 static passes, 53 correction-required and 2 unresolved effects (`sigilFx`, `sigilHit`). The canonical identity requires a long purple coat with gold details, cream lining, white gloves and dark trousers. Most findings concern pose, finish and sequence continuity rather than proven identity changes. The new three-pose jab integration is covered by its own report; this full-roster inventory still carries pre-integration jab hashes and must be regenerated after art freeze. Runtime, collision and transformation reviews remain open.

### Independent Suzume family audit — 2026-11-12

Suzume: 83 families reviewed against `04_special.png` SHA-256 `3621301628480a47e63dd77a0084d766f69da01d44bb419a532b73be0f9885be`; the initial report found 23 static passes, 55 correction-required and 5 unresolved. Two further corrections (`hitCrouch`, `hitL1`) are now independently reviewed and integrated, so the adjusted dispositions are 26 pass, 52 correction-required, 5 unresolved pending manifest refresh. Most failures use masked purple/red ninja variants instead of canonical Suzume. The `hitCrouch` and `hitL1` integrations retain the canonical kunai detail; full hit/guard family still has unresolved legacy frames. Runtime and PR gates remain open.

### Independent playable-roster family audit roll-up — 2026-11-12

Thirteen of the 16 playable native fighters have independent family-level reports: 1,052 ordered animation families across Akio, Bruno, Jin, Kaito, Lian, Oboro, Shion, Sylene, Marlow, Daichi, Garan, Haruki and Suzume. Pre-promotion reviewer dispositions totaled 523 static-pass, 470 correction-required and 59 unresolved families. After the Haruki jab and Suzume hitCrouch static promotions, the adjusted subset is 525/468/59. These remain static family dispositions; they do not clear the 1,703-row consolidated inventory, replace exact per-frame source review, or imply runtime/transition approval. Kaname, Valdric and Volta audits are still underway. Five NPC clone packs still need independent source/recolor review.

### Independent NPC clone/source audit — 2026-11-12

Read-only audit of the five story NPC packs found source mappings: brute→Bruno, colossus→Daichi, phantom→Jin, razor→Suzume, spitfire→Akio. `npc_brute`: 41 static passes, 36 correction-required, 2 unresolved effects. `npc_colossus`: 81 families are unresolved by design in the conservative NPC visual inventory, not because assets are missing. `spriteSet=daichi` resolves to Daichi body frames and the NPC applies its own palettes; review must render the final recolor before approving those families. `npc_phantom`: 77 correction-required and 3 unresolved effects; inspect clone/recolor compatibility separately from Jin source residues. `npc_razor`: 78 static passes and 5 unresolved effects. `npc_spitfire`: 62 static passes, 14 correction-required and 5 unresolved, including an unresolved idle. These are static classifications; prior story runtime captures only prove NPC rendering/integration, not every sprite family or recolor.

### Playable transformation visual cross-review — 2026-11-12

All 16 playable `powerUp` transitions have three distinct frames and passed static identity, outfit and color comparison at native/2× size; the 48 source hashes are in `transformation-static-audit/powerup-source-review.json`. Separate post-transform captures prove only each stable transformed idle state. They do not validate the transition at runtime, attacks while transformed, or collision behavior; form-move sprite families are being audited separately.

### Suzume crouching-hit correction — 2026-11-12

Integrated the v2 `hitCrouch.png` with canonical kunai, preserving the original and `meta.json` in `integrated/suzume-hitCrouch-before/`. The white blade in the old sprite is confirmed as Suzume's declared kunai, not hit flash: the move leaves the hands open, and the renderer draws the default weapon unless explicitly disabled. The candidate keeps the identity-correct long purple hair, red bow, face mask and violet outfit. Static gates pass at 400×400, alpha binary, axis (200,392), refHeight 320, 47 opaque colors in the shared palette, ΔE 0.12204/0.13, with no off-palette or partial-alpha pixels. This resolves the kunai concern and the `hitCrouch` image only; `hitL1/2/3`, `guard`, `guardHit`, `airGuard`, and `airGuardHit` remain open. Runtime remains pending. See `suzume-hitCrouch-integration-20261112.json`.

### Transformed-form attack sprite audit — 2026-11-12

Resolved `forms[].moves[]` for all 16 playable characters. Form attacks reference the character's current base sprite families with palette/form declarations; no cross-character identity switch was found. This does not make their attacks visually clean: any base family classified correction-required remains so in the transformed form. RIG poses, effects and projectiles remain inconclusive; the full form attack mapping is in `transformed-form-attacks-static-inventory-20261112.md`. No runtime attack/transition approval.

## Current review authority and coverage — epoch NATIVE-ROSTER-2026-11-12-E

| Record | Current value |
|---|---|
| Artifact | `ultimate-mugen-roster-quality`: native combat PNGs, character JSON, metadata, generation/runtime support code, and review evidence |
| Mode / round | Improve; correction pass 1; no clean streak (asset promotions reset convergence) |
| Scope | All 16 playable native characters, five story NPC clone packs, their declared and resolved animation/form families, and final evidence for requested PR |
| Exclusions | Imported M.U.G.E.N. files, portraits, stages, unrelated product systems |
| Protected surfaces | Canonical identity/appearance, pixel-cluster finish, palette, canvas/floor/axis, attacks/hit timing, animation/control recovery, form declarations |
| Privacy/licence | Local repository assets only; no external sprite sources used |
| Host / clarification | root / root |
| Artifact mutation | root owns active promotions; workers produce only isolated candidates in explicitly assigned paths |
| Runtime | root; process-isolated capture from frozen art after the current correction batch |
| Review | five role-separated/independent character or candidate auditors; reviewer evidence does not imply user appearance acceptance |
| Promotion execution | root; PR has not been opened |
| File layer | Additive/copy-on-write candidate promotion; every current promotion has before-copy, manifest/hash and clean image reopen; older pre-existing metadata repairs have a historical backup gap recorded above |
| Runtime layer | Required and pending for the new corrections; final work must use process-isolated disposal and prove clean-process captures from frozen inputs |
| Acceptance authority | User-provided AGENTS.md and final appearance approval by the user |
| Claim ceiling | Current findings are static unless a specific report says otherwise; full visual standardization and all attack/form runtime are not established |
| Bound | Four material-finding rounds or 90 minutes per frozen review packet; current art corrections create a new epoch after promotion |
| Stop/reopen | Keep all material unresolved items in the ledger; after each promotion refresh hashes and begin a new static review pass; do not open PR until corrected families, tests, runtime evidence and PR visuals are reviewable |

### Coverage matrix

| Surface | Required | Evidence / result | Reviewer | State |
|---|---|---|---|---|
| Sprite canvas, alpha, palette, axes | Yes | Existing roster structural audits plus per-pilot strict QA; report is pre-promotion for most of roster | Automated + root visual | Partial; refresh after art freeze |
| Identity, pose and family continuity | Yes | Independent static reviews cover 13/16 playable packs; pilots cover active corrections | Character reviewers | Open: 470 correction rows in independent subset; 3 characters plus NPC renders remain |
| Attacks, hit timing and recovery | Yes | Low-kick runtime captures and source-level frame mappings; several changed sequences not run | Static + prior runtime | Open |
| Transformations | Yes | All 16 `powerUp` frames static-reviewed; stable transformed idle captures; form moves source-mapped | Independent static review | Transition runtime and form attack runtime open |
| NPC clones/recolors | Yes | Source mapping for five NPCs; Colossus final palette proof and Phantom/Spitfire family fixes remain | Static reviewer | Open |
| Privacy/provenance | Yes | Local-only assets; no external source used | Root | Pass for current local work; PR outbound audit pending |
| Final runtime/build/test gates | Yes | Existing low-kick and NPC captures do not cover changed art; no final test/browser run in this pass | Root | Pending after art freeze |
| PR evidence and publication | Yes | Prior visual evidence exists; current PR package not frozen; automation/reachability audit not performed | Root | Not ready / not opened |

### Independent Volta family audit — 2026-11-12

Volta: 81 families reviewed against `04_special.png`; 35 static passes, 42 correction-required, 4 unresolved procedural effects (`boltFx`, `boltHit`, `coreFx`, `coreHit`). The active robot identity remains recognizable; many open families use an older pale body finish and pose flow. Nine groups were misclassified `static_pass` in the consolidated report: `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `hop`, `overclock`, `super`, `relay`, `relayUlt`. The integrated jab palette repair is exact, but the page manifest and contact sheets are stale for jab0/jab1 and omit jab2. See `volta-full-family-independent-review-20261112.md`. Static only; runtime open.


### Suzume light-hit correction — 2026-11-12

Integrated canonical `hitL1.png` from the independently reviewed v2 candidate; original preserved in `integrated/suzume-hitL1-before/`. The frame keeps Suzume's face mask, violet outfit, purple ponytail/red bow and one canonical front-hand kunai confirmed by the actual hitLowLight pose and rig render. QA passes: 400×400 RGBA, binary alpha, axis (200,392), visible floor 391, 46 used colors in the shared 48-entry palette, ΔE 0.12204/0.13, no off-palette/partial-alpha pixels. Contact sheets show the frame against canonical, old target, and hitL2/3. Runtime and neighboring hitL2/3 remain open. Evidence: `suzume-hitL1-integration-20261112.json`.


### Garan frame-level pilot correction — 2026-11-12

The Garan `cjab` family itself is `static_pass`; its `cjab0.png` is canonical and must not be replaced. Frame-level source review found the wrong-identity body images are `jumpTop.png` and `jumpDown.png`; `jumpTop` is reused by `jumpUp`, `jumpBack`, `jumpFwd`, `akick` and `airLowKick`, making it the higher-leverage correction. The pilot has been redirected to `jumpTop`; prior cjab candidates remain isolated and rejected by fit/pose, with active asset unchanged.

### Kaname cjab1 v9 adjudication — 2026-11-12

The v9 candidate passed technical registration and palette gates, but the revised katana points upper-left while Kaname faces right, so the pose no longer reads as the active forward crouching jab. It remains unapproved. Preserve the original cjab1 until a full-length forward-right composition fits the 400×400 canvas at canonical scale; if it cannot, keep the family unresolved rather than reversing direction or shortening the blade.


### Suzume medium-hit correction — 2026-11-12

Integrated canonical `hitL2.png` from the independent v2 candidate; original preserved in `integrated/suzume-hitL2-before/`. Its raised/forward head and stronger hit recoil read distinctly after corrected `hitL1`, with one canonical front-hand kunai confirmed by the actual hitLowMedium pose/rig. QA passes: 400×400 RGBA, binary alpha, axis (200,392), floor 391, 47 used colors in the shared 48-entry palette, ΔE 0.07771/0.13, no off-palette or partial-alpha pixels. The active `hitLowMedium` timing remains 3 ticks of hitL2, 8 of hitL1, then 3 idle ticks. Runtime remains open; hitL3 and the other old guard/hit frames are not resolved. See `suzume-hitL2-integration-20261112.json`.

### Garan shared jumpTop correction — 2026-11-12

Integrated the independently reviewed canonical candidate for the shared `jumpTop.png` source; original preserved in `integrated/garan-jumpTop-before/`. The source audit showed `cjab0.png` and `ajab.png` were already canonical, so those false-positive targets were left unchanged. `jumpTop` supplies `jumpUp`, `jumpFwd`, `jumpBack`, `ajab`, `akick` and `airLowKick`. The candidate passes 400×400, binary alpha, axis (200,392), floor 391, 273px target body height, 46 colors in a shared 48-color palette, ΔE 0.086491/0.13, and no off-palette/partial-alpha pixels. It restores the red oni, white mane, yellow horns, purple hakama and gold rope. `jumpDown` and the other affected sequence frames remain open; runtime is pending. See `garan-jumpTop-integration-20261112.json`.


### Kaname cjab1 v10 adjudication — 2026-11-12

The forward-right crouching strike candidate kept blade length and body scale but exceeded the right edge by 106px when registered at 97% scale; a full-scale probe touched the output boundary. It is rejected/unresolved. Do not reduce character scale, shorten the canonical katana or use a backward-pointing pose to make it fit. The active `cjab1.png` remains untouched; v10 previews and measurements are in `candidates/kaname-cjab1-pilot/pipeline-v10/`. Work moved to a separate high-reuse crouch-settle pose while `cjab1` remains open.


### Suzume hitL3 candidate review — 2026-11-12

The first candidate for `hitL3` is rejected for pose mismatch despite passing palette, canvas and alpha gates. The actual `hitLowHeavy` rig/source uses an arched-back recoil with legs braced; the candidate instead crouches and leans forward, repeating a different phase and breaking progression after corrected hitL1/hitL2. The active file is unchanged. A pose-locked v2 is in progress; no promotion is allowed until the back-arched phase reads correctly in native/2× review.
