# Vida vermelha, tontura e pontos de defesa

Todo lutador carrega três sistemas de vida do Ikemen GO ao lado de `Life`. Eles ficam sempre ativos; um personagem que nunca os lê só ganha o estado extra.

## Vida vermelha

`RedLife` é a parte recuperável da barra de vida. Nunca fica abaixo de `Life` e nunca passa de `LifeMax`.

- Um golpe subtrai o valor `redlife` do HitDef de `RedLife`, então a barra vermelha cresce por cima da vida real. Um golpe defendido usa o segundo valor de `redlife`.
- `LifeAdd`, `LifeSet`, `RedLifeAdd` e `RedLifeSet` respeitam a regra acima. Quando `Life` passa de `RedLife`, `RedLife` acompanha.
- Um novo round volta `RedLife` para `LifeMax`.
- `NoRedLifeDamage` impede que golpes a alterem.

## Pontos de tontura

`DizzyPoints` começam em `DizzyPointsMax` (a constante `data.dizzypoints`, `data.life` quando ausente).

- Um golpe não defendido subtrai o valor `dizzypoints` do HitDef, multiplicado pelo multiplicador de defesa do alvo.
- Em 0 o alvo entra no estado comum 5300 (tontura) com `Dizzy` ligado e sem controle. Alvos no ar ou caindo não ficam tontos.
- Quando o estado de tontura termina e o alvo sai do estado de golpe, `Dizzy` desliga e os pontos enchem de novo.
- `NoDizzyPointsDamage` impede que golpes reduzam os pontos.

## Pontos de defesa

`GuardPoints` começam em `GuardPointsMax` (`data.guardpoints`, `data.life` quando ausente).

- Um golpe defendido subtrai o valor `guardpoints` do HitDef, multiplicado pelo multiplicador de defesa do alvo.
- Em 0 o alvo entra no estado comum 5310 (quebra de defesa) com `GuardBreak` ligado e sem controle.
- Quando o estado termina e o alvo sai do estado de golpe, `GuardBreak` desliga e os pontos enchem de novo.
- `NoGuardPointsDamage` impede que golpes defendidos reduzam os pontos.

## Estados comuns

O `common1.cns` traz os estados 5300 (tontura, animação 5300 depois 5000) e 5310 (quebra de defesa, animação 5310, depois 150, depois 5000). O personagem pode sobrescrever qualquer um no próprio CNS. Sem nenhum dos dois estados, o sistema correspondente só acompanha os números.

## Padrões do HitDef

Quando um HitDef omite `redlife`, `dizzypoints` ou `guardpoints`, a engine os deriva do dano com os multiplicadores do `[Constants]` do personagem:

| Constante | Padrão | Padrão de super (`attr` com `H`) |
|---|---|---|
| `lifetoredlifemul` | 0.75 | 0.75 |
| `lifetodizzypointsmul` | 1.8 | 0 |
| `lifetoguardpointsmul` | 1.5 | -0.33 |

As chaves se escrevem `default.lifetoredlifemul` e `super.lifetoredlifemul` numa seção `[Constants]`. Um valor negativo de pontos de defesa devolve pontos, então um super recupera um pouco de defesa ao acertar. `score = <acerto>, <defesa>` também é lido pelo HitDef e guardado para `GetHitVar(score)`.

## Triggers e controllers

`RedLife`, `Dizzy`, `DizzyPoints`, `DizzyPointsMax`, `GuardBreak`, `GuardPoints`, `GuardPointsMax`, `GetHitVar(redlife)`, `GetHitVar(dizzypoints)` e `GetHitVar(guardpoints)` leem esses valores. `RedLifeSet`, `RedLifeAdd`, `DizzyPointsSet`, `DizzyPointsAdd`, `DizzySet`, `GuardPointsSet` e `GuardPointsAdd` os alteram (veja [Extensões do Ikemen](Ikemen-Extensions.md)).
