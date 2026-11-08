# Passives

Native (UMF) fighters opt into passive systems through the `passives` object of their JSON. Nothing is hardcoded to a fighter name: M.U.G.E.N. characters and native fighters without these keys are not affected.

```json
"passives": {
  "meterRegen": 0.35,
  "lifeRegen": 0.04,
  "rage": { "maxMultiplier": 4.0, "exponent": 1.0, "startRatio": 1.0, "fullRatio": 0.01 },
  "stacks": { "damageGain": 0.07, "defenceGain": 0.015, "speedGain": 0.015, "maxStacks": 10, "decayTicks": 420, "resetOnRound": true, "specials": true, "supers": true }
}
```

## Regeneration

`meterRegen` adds power every tick and `lifeRegen` adds life every tick while a round is in progress (compiled into `[Statedef -2]`).

## Rage (damage grows as life drops)

Used by Bruno. With `r = life / lifemax`:

```
progress   = clamp((startRatio - r) / (startRatio - fullRatio), 0, 1)
multiplier = 1 + (maxMultiplier - 1) * progress ^ exponent
```

| Key | Default | Range | Meaning |
|---|---|---|---|
| `maxMultiplier` | 4.0 | 1 to 6 | Damage multiplier reached at `fullRatio` |
| `exponent` | 1.0 | 0.25 to 4 | 1 is linear, above 1 stays low longer then ramps, below 1 ramps early |
| `startRatio` | 1.0 | 0.05 to 1 | Life share where the bonus starts (1.0 = from full life) |
| `fullRatio` | 0.01 | 0 to `startRatio` - 0.01 | Life share where the maximum is reached |

With the defaults the curve is 1.0x at 100%, about 2.5x at 50%, about 3.7x at 10% and 4.0x at 1%. It is continuous: no steps.

## Ability stacks (stronger with every special)

Used by Sylene. Each time the fighter enters a special state (1000 to 1999) or a super state (3000 to 3899), including the air variants (+20000), it gains one stack up to `maxStacks` (1 to 30). Normals, fly, teleport and super speed do not stack.

| Key | Default | Effect per stack |
|---|---|---|
| `damageGain` | 0.05 | Damage multiplier `1 + damageGain * stacks` |
| `defenceGain` | 0 | Defence multiplier `1 + defenceGain * stacks` (capped at 3x) |
| `speedGain` | 0 | Horizontal movement speed `1 + speedGain * stacks` (capped at 1.5x, not applied while being hit) |
| `maxStacks` | 10 | Stack ceiling |
| `decayTicks` | 0 | When above 0, one stack is lost after this many ticks without gaining one |
| `resetOnRound` | true | Stacks are cleared when a round starts |
| `specials`, `supers` | true | Which move kinds count |

## Composition rule and cap

```
effective attack  = min(attackMul * rage * (1 + damageGain * stacks), max(attackMul, 8.0))
effective defence = defenceMul * superDefenceMul * (1 + defenceGain * stacks)
```

`attackMul` already contains the base `stats.attack` and the awakened or ascended form multipliers (PR #415), so every source multiplies. The global ceiling is 8x the base damage; a fighter whose own `attackMul` is already higher (for example a M.U.G.E.N. `AttackMulSet`) keeps it. Without a `rage` or `stacks` key the original multipliers are used unchanged.

## HUD and effects

- A thin strip along the bottom of the health bar shows rage progress (orange to gold) and a row of pips shows stacks.
- The awakened-form aura is reused: it appears once rage or stacks pass 8% and grows with the larger of the two. Rage alone tints it red.
- Stack state is part of the netplay checksum.

## Code

`crates/engine/src/passives.rs` (rules, state, `Fighter::effective_attack`), `crates/engine/src/native/passive_spec.rs` (JSON), `src/hud/passive_meter.rs`, `src/passive_fx.rs`. Tests: `crates/engine/tests/passives.rs`. Screenshots: `UME_TOUR=passives ultimate-mugen --shots <dir>`.
