# Signature Meter

[Português (BR)](pt-BR/Medidor-de-Assinatura.md)

Each native fighter can declare a `signature` block under `passives` in its JSON. It is a per-fighter meter that changes how the fighter plays, on top of rage, stacks and enrage time.

```json
"passives": {
  "signature": {
    "name": "Storm",
    "max": 100,
    "gainHit": 6,
    "gainSpecial": 12,
    "gainHurt": 3,
    "decayDelaySeconds": 2.5,
    "decayPerSecond": 14,
    "drainOnSuper": true,
    "riskPerSecond": 0,
    "tiers": [
      { "at": 34, "attack": 1.08, "speed": 1.04 },
      { "at": 100, "attack": 1.32, "speed": 1.12, "defence": 0.9 }
    ]
  }
}
```

| Key | Meaning |
| --- | --- |
| `max` | Meter size, 1 to 200. |
| `gainHit`, `gainSpecial`, `gainHurt` | Gain when a move connects, a special or super starts, and when hit. |
| `decayDelaySeconds`, `decayPerSecond` | Idle time before the meter drains, and the drain rate. |
| `drainOnSuper` | Spends the meter when a super ends, so the super benefits from it. |
| `riskPerSecond` | Life lost per second while the meter is full. Never kills. |
| `tiers` | Up to four steps. The highest reached applies `attack` (0.5 to 2), `defence` (0.5 to 1.5) and `speed` (0.8 to 1.25). |

Attack still goes through the global cap in [Passives](Passives.md). The meter resets each round and shows as a strip under the life bar.

Current fighters: **Kaito** (Storm, builds and speeds him up) and **Volta** (Overload, strong at full meter but costs life).
