# Quadros de efeito

Os lutadores são desenhados apenas por sprites PNG em pixel art (`assets/chars/sprites/<id>/`). A única arte desenhada pelo motor são os efeitos de projétil e faíscas abaixo.

Um quadro com `effect` desenha um efeito no lugar do corpo. Use em projéteis e faíscas.

```json
{ "effect": { "type": "slash", "size": 26, "color": "#ff3a5c", "color2": "#ffd0d8", "phase": 0 }, "ticks": 3 }
```

| `type` | Formato |
|---|---|
| `orb` (padrão), `burst` | Bola de energia / explosão |
| `dragon` | Dragão de energia voltado para frente, com corpo em movimento |
| `beam` (`length`) | Raio horizontal |
| `pillar` (`length`) | Coluna vertical |
| `shockwave` | Onda no chão |
| `slash`, `wave`, `water` | Corte em arco |
| `kunai`, `shuriken` | Arremessáveis |
| `ice`, `fire`, `darkfire`, `lightning`, `petal` | Elementos |

Sem `color`, o efeito usa a cor `energy` da paleta. A caixa de acerto do efeito é gerada sozinha a partir do formato.
