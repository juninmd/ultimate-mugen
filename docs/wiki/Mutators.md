# Match Mutators

Mutators change the rules of one match without touching fighter data. The first mutator is **gravity**.

| Field | Default | Range | Effect |
|---|---|---|---|
| `gravity` | `1.0` | `0.25` to `4.0` | Scales the fall of airborne fighters and the drop of grounded fighters that are left in the air |

Values outside the range, or not finite, are rejected by `Mutators::with_gravity` and return `None`, so a bad value never reaches the simulation.

Scope in this version: the scale applies to the fighter's own `movement.yaccel`. Hit gravity from HitDef (`yaccel` on the hit) is not scaled yet.

Both peers of a rollback match must use the same mutators. They are set on the `Match` before the first tick (`m.mutators = ...`). The default `1.0` is a no-op, so existing matches are unchanged.

Code: `crates/engine/src/mutators.rs`, applied in `physics_step` (`crates/engine/src/matchs.rs`). Tests: `crates/engine/tests/mutators.rs`.
