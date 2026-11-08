# Passivos

Lutadores nativos (UMF) ativam passivos pelo objeto `passives` do JSON. Nada é fixo por nome: personagens M.U.G.E.N. e nativos sem essas chaves não são afetados.

```json
"passives": {
  "meterRegen": 0.35,
  "lifeRegen": 0.04,
  "rage": { "maxMultiplier": 4.0, "exponent": 1.0, "startRatio": 1.0, "fullRatio": 0.01 },
  "stacks": { "damageGain": 0.07, "defenceGain": 0.015, "speedGain": 0.015, "maxStacks": 10, "decayTicks": 420, "resetOnRound": true, "specials": true, "supers": true }
}
```

## Fúria (dano cresce conforme a vida cai)

Usada pelo Bruno. Com `r = vida / vidaMax`:

```
progresso     = clamp((startRatio - r) / (startRatio - fullRatio), 0, 1)
multiplicador = 1 + (maxMultiplier - 1) * progresso ^ exponent
```

Com os padrões: 1,0x com vida cheia, cerca de 2,5x com 50%, 3,7x com 10% e 4,0x com 1%. A curva é contínua, sem degraus. `exponent` 1 é linear, acima de 1 demora a subir, abaixo de 1 sobe cedo. `startRatio` define de quantos % de vida o bônus começa e `fullRatio` onde atinge o máximo.

## Acúmulos de habilidade

Usados pela Sylene. Cada vez que o lutador entra em um golpe especial (estados 1000 a 1999) ou super (3000 a 3899), inclusive as variantes aéreas, ganha um acúmulo até `maxStacks` (1 a 30). Golpes normais, voo, teletransporte e supervelocidade não contam.

- `damageGain`: dano `1 + damageGain * acúmulos`.
- `defenceGain`: defesa `1 + defenceGain * acúmulos` (máximo 3x).
- `speedGain`: velocidade horizontal `1 + speedGain * acúmulos` (máximo 1,5x, não vale enquanto apanha).
- `decayTicks`: acima de 0, perde um acúmulo após esse tempo sem ganhar outro.
- `resetOnRound`: zera os acúmulos a cada round (padrão: sim).

## Regra de composição e limite

```
ataque efetivo = min(attackMul * fúria * (1 + damageGain * acúmulos), max(attackMul, 8.0))
defesa efetiva = defenceMul * superDefenceMul * (1 + defenceGain * acúmulos)
```

`attackMul` já inclui `stats.attack` e os multiplicadores das formas despertada e ascendida (PR #415), então tudo se multiplica. O teto global é 8x o dano base. Sem `rage` nem `stacks` os multiplicadores originais ficam intactos.

## HUD e efeitos

Uma faixa fina na base da barra de vida mostra o progresso da fúria e fileira de marcadores mostra os acúmulos. A aura das formas é reaproveitada e cresce com a fúria ou os acúmulos.
