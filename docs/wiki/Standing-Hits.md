# Standing Hits (knockdown rule)

Fighters used to fall after about one grounded hit in seven. The rule below lets light and medium hits push the defender back while they stay on their feet; strong hits, supers and long combos still knock down.

## Rule

For a hit on a **grounded, living** defender whose HitDef asks for `fall`, the engine keeps the defender standing when all of these hold:

- the hit takes less than the threshold of the defender's maximum life (default 16%);
- the combo total including this hit is under 3x the threshold (so a long string still ends in a knockdown);
- the attacker is covered by the scope (default: native fighters only);
- the attack is not a hyper (`H*` attr), has no `down.bounce`, `wall.bounce` or trip `ground.type`, no `p2stateno`, and no `forcenofall`;
- the hit is not the killing blow.

When it applies, the defender takes the stand-hit states (`5000`/`5010`, then slide `5001`/`5011`), the authored vertical velocity is dropped and the horizontal push grows up to 75% (the closer the hit is to the threshold, the bigger the push). Hits on airborne defenders (juggles) are never touched, so air combos behave exactly as before.

Code: `crates/engine/src/matchs/knockdown.rs` (`Match::standing_push`), wired in `Match::apply_hit`.

## Scope and compatibility

| Scope | Covers | Why |
|---|---|---|
| Native (default) | hits from UMF native fighters | their `fall` is derived from `level: heavy` / `launch`, not authored per move |
| Everyone | also M.U.G.E.N fighters | M.U.G.E.N HitDefs set `fall` on purpose (sweeps, launchers); opt-in only |
| Classic | nobody | exact M.U.G.E.N behaviour |

Scope follows the **attacker** because the HitDef is the attacker's data; the defender's own states (`5000`...) work for both kinds.

## Setting

Settings > Gameplay > *Knockdowns on hit* (`knockdown` in `ultimate-mugen.cfg`): Classic, Native 8%, Native 16% (default), Native 24%, Everyone 16%. A higher threshold means fewer knockdowns. The value is stored in replays (`knockdown_scope`, `knockdown_threshold`; replays saved before it existed replay as Classic) and netplay uses the default preset.

## Measuring

`ume-tools knockdown-rate --roster akio,bruno,... --frames 5400 --rule native --threshold 16` runs AI vs AI over the roster and prints, per damage bucket, how many hits on a grounded defender ended in a knockdown. Results are in the PR that introduced the rule.
