# Red life, dizzy and guard points

Every fighter carries three Ikemen GO life systems next to `Life`. They are always on; a character that never reads them is unaffected apart from the extra bars of state.

## Red life

`RedLife` is the recoverable part of the life bar. It never drops below `Life` and never rises above `LifeMax`.

- A hit subtracts the HitDef `redlife` value from `RedLife`, so the red bar grows over the real life bar. A guarded hit uses the second `redlife` value.
- `LifeAdd`, `LifeSet`, `RedLifeAdd` and `RedLifeSet` keep the rule above. When `Life` rises over `RedLife`, `RedLife` follows it.
- A new round resets `RedLife` to `LifeMax`.
- `NoRedLifeDamage` stops hits from changing it.

## Dizzy points

`DizzyPoints` start at `DizzyPointsMax` (the `data.dizzypoints` constant, `data.life` when absent).

- A hit that is not guarded subtracts the HitDef `dizzypoints` value, multiplied by the defender's defence multiplier.
- At 0 the defender enters common state 5300 (dizzy) with `Dizzy` set and control removed. Airborne or falling defenders do not become dizzy.
- When the dizzy state ends and the defender is out of hit state, `Dizzy` clears and the points refill.
- `NoDizzyPointsDamage` stops hits from lowering the points.

## Guard points

`GuardPoints` start at `GuardPointsMax` (`data.guardpoints`, `data.life` when absent).

- A guarded hit subtracts the HitDef `guardpoints` value, multiplied by the defender's defence multiplier.
- At 0 the defender enters common state 5310 (guard break) with `GuardBreak` set and control removed.
- When the state ends and the defender is out of hit state, `GuardBreak` clears and the points refill.
- `NoGuardPointsDamage` stops guarded hits from lowering the points.

## Common states

`common1.cns` ships states 5300 (dizzy, animation 5300 then 5000) and 5310 (guard break, animation 5310 then 150 then 5000). A character may override either state in its own CNS. When neither state exists the matching system only tracks the numbers.

## HitDef defaults

When a HitDef omits `redlife`, `dizzypoints` or `guardpoints`, the engine derives them from the damage with the character's `[Constants]` multipliers:

| Constant | Default | Super default (`attr` has `H`) |
|---|---|---|
| `lifetoredlifemul` | 0.75 | 0.75 |
| `lifetodizzypointsmul` | 1.8 | 0 |
| `lifetoguardpointsmul` | 1.5 | -0.33 |

The keys are written `default.lifetoredlifemul` and `super.lifetoredlifemul` in a `[Constants]` section. A negative guard-points value gives guard points back, so a super move restores a little guard on contact. `score = <hit>, <guard>` is also read by HitDef and stored for `GetHitVar(score)`.

## Triggers and controllers

`RedLife`, `Dizzy`, `DizzyPoints`, `DizzyPointsMax`, `GuardBreak`, `GuardPoints`, `GuardPointsMax`, `GetHitVar(redlife)`, `GetHitVar(dizzypoints)` and `GetHitVar(guardpoints)` read these values. `RedLifeSet`, `RedLifeAdd`, `DizzyPointsSet`, `DizzyPointsAdd`, `DizzySet`, `GuardPointsSet` and `GuardPointsAdd` change them (see [Ikemen extensions](Ikemen-Extensions.md)).
