# Rivalries

[Português (BR)](pt-BR/Rivalidades.md)

A native fighter can declare `rivals` in its JSON, keyed by the opponent's fighter id. Each entry can change what the fighter says at the round start and give a small attack bonus in that matchup.

```json
"rivals": {
  "akio": { "intro": "Akio. Your star shines, but the storm asks no permission.", "taunt": "..." },
  "garan": { "attack": 1.06 }
}
```

| Key | Meaning |
| --- | --- |
| `intro` | First-speaker line at the round start, used instead of the generic `intro`. |
| `taunt` | Answer line, used instead of the generic `taunt`. |
| `attack` | Attack multiplier against that opponent only, clamped to 0.9 to 1.15. |

The bonus multiplies the fighter's attack after passives, so it never exceeds the global cap in [Passives](Passives.md). Fighters without `rivals` behave as before.

Current pairs follow [Lore](Lore.md): Kaito and Akio (friendly rivals), Suzume against Oboro, Sylene against Valdric and Kaname, Bruno and Marlow against Valdric, Shion against Garan.
