# Effect frames

Fighters are drawn only from pixel-art PNG sprites (`assets/chars/sprites/<id>/`). The only engine-drawn art left is the projectile and spark effects below.

A frame with `effect` draws an effect in place of the body. Use it for projectiles and sparks.

```json
{ "effect": { "type": "slash", "size": 26, "color": "#ff3a5c", "color2": "#ffd0d8", "phase": 0 }, "ticks": 3 }
```

| `type` | Shape |
|---|---|
| `orb` (default), `burst` | Energy ball / explosion |
| `dragon` | Forward-facing energy dragon with a trailing body |
| `beam` (`length`) | Horizontal ray |
| `pillar` (`length`) | Vertical column |
| `shockwave` | Ground wave |
| `slash`, `wave`, `water` | Arc cut |
| `kunai`, `shuriken` | Throwables |
| `ice`, `fire`, `darkfire`, `lightning`, `petal` | Elements |

Without `color`, the effect uses the palette's `energy` color. The effect hitbox is generated from its shape.
