# Kit Inputs

[Português (BR)](pt-BR/Entradas-de-Kit.md)

Native fighters no longer share one input set. Each archetype swaps some quarter-circle motions for inputs that fit its style. The move list in the pause menu shows the real input of every move.

| Fighter | Move | Input | Style |
| --- | --- | --- | --- |
| Valdric | Sovereign | hold back 40 ticks, forward + y | Charge (zone control) |
| Volta | Tesla | hold back 40 ticks, forward + y | Charge (zoner) |
| Bruno | Lariat | 360° motion + y | Grappler, command throw |
| Daichi | Aura Rush | 360° motion + x | Grappler rush |

Charge motions use the `~40$B, F, x` syntax and work in the training mode, versus and online. Air variants of a move need a motion that can be done in the air, so charge inputs are kept for ground-only moves.

## Air mobility

`stats.airDashes` sets how many air dashes a fighter has before landing (default 1). Rushdown and aerial fighters get more, grapplers fewer.

| Fighter | Air dashes |
| --- | --- |
| Suzume, Jin, Akio, Lian, Kaito | 2 |
| Bruno | 0 |
| Everyone else | 1 |

## Rekka follow-ups

A move can only start from another move by declaring `rekka` in the character JSON:

```json
{"id":"gust_cross","name":"Gust Crossup","input":"b","kind":"special","anim":"mirage","rekka":{"after":["gustdash"],"window":[6,20]}}
```

`after` lists the source move ids, `window` is the range of state time (ticks inside the source move, hit pause does not count) where the follow-up input is accepted, and `contact: true` also requires the source to hit or be guarded. Without it the follow-up works on whiff too. The follow-up never starts from neutral, so it may reuse a plain button already used by a normal. Code: `crates/engine/src/native/rekka.rs`.

In use: Lian (Flowing Silk Kicks, then `y`), Kaito (Storm Rush, then `x`) and Jin (Gust Dash, then `b`).

## Distance and state requirements

`requires` also accepts `distance` (pixels between the bodies, `min` and/or `max`) and `enemyState` (`air`, `ground`, `crouch`, `down`, `hit` or `attack`). Two moves can share one input and the engine picks by situation, as long as the conditions do not overlap. A variant must come before the move it overrides in the `moves` list, because the first move that fits wins.

In use: Daichi (Landslide Point Blank within 55 pixels, otherwise Landslide Heel) and Bruno (Bull Catch against an airborne enemy, otherwise Bulldozer Rush).
