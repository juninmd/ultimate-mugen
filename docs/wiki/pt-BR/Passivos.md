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

## Fúria por tempo (mais forte quanto mais a luta demora)

Usada pelo Garan, o Oni das Cinzas. O dano cresce com o tempo de luta no round atual. O contador só roda com o round ativo (não na introdução, KO ou vitória), para quando o lutador morre e reinicia a cada round.

```
progresso = clamp((segundosDeLuta - startSeconds) / rampSeconds, 0, 1)
multiplicador = 1 + (maxMultiplier - 1) * progresso ^ exponent
```

- `maxMultiplier` (padrão 5, de 1 a 6): dano máximo no fim da rampa.
- `startSeconds` (padrão 0): segundos de luta antes do bônus começar.
- `rampSeconds` (padrão 60): segundos do início até o máximo.
- `exponent` (padrão 1): 1 linear, acima de 1 sobe tarde, abaixo de 1 sobe cedo.

O Garan usa 15 s de carência e 105 s de rampa: 1x até 0:15, 3x em 1:07 e 5x a partir de 2:00.

## Regra de composição e limite

```
ataque efetivo = min(attackMul * fúria * tempo * (1 + damageGain * acúmulos), max(attackMul, 8.0))
defesa efetiva = defenceMul * superDefenceMul * (1 + defenceGain * acúmulos)
```

`attackMul` já inclui `stats.attack` e os multiplicadores das formas despertada e ascendida (PR #415), então tudo se multiplica. O teto global é 8x o dano base; os 5x do tempo sozinhos nunca são cortados, e combinados com fúria máxima vale o teto de 8x. Sem `rage` nem `stacks` os multiplicadores originais ficam intactos.

## HUD e efeitos

Uma segunda faixa (violeta para vermelho) acima da faixa de fúria mostra a fúria por tempo. Uma faixa fina na base da barra de vida mostra o progresso da fúria e fileira de marcadores mostra os acúmulos. A aura das formas é reaproveitada e cresce com a fúria ou os acúmulos.
