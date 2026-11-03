# Criar personagem UMF

UMF (*Ultimate Mugen Format*) descreve um lutador inteiro em **um arquivo JSON**. O compilador (`crates/engine/src/native/compile.rs`) gera comandos, estados, HitDefs, projéteis, super pause, cancelamentos e hitboxes. O resultado roda na mesma máquina de estados dos personagens M.U.G.E.N.

## Passo a passo

1. Copie um lutador pronto como base: `cp assets/chars/kaito.json assets/chars/meu.json`.
2. Troque `id` (único, minúsculo, sem espaço), `name`, `series`, `author` e `color`.
3. Opcional: coloque o `id` em `assets/chars/index.json` para definir a ordem na seleção. Os ausentes entram em ordem alfabética depois.
4. Rode `cargo run --release`. O `build.rs` embute todos os `.json` da pasta no executável.
5. Teste no **Treino** com `F2` ligado para ver hitboxes e hurtboxes.

Carregar UMF de uma pasta sem recompilar: **planejado** (o registro já tem `add_native_json`, falta ligar à pasta `mugen/`).

## Esqueleto do arquivo

```json
{
  "format": "umf/1",
  "id": "meu",
  "name": "Meu Lutador",
  "series": "Minha Série",
  "author": "você",
  "description": "Texto da tela de seleção",
  "color": "#e85d2a",
  "winQuote": "Frase de vitória",
  "intro": "Fala no início do round 1",
  "taunt": "Resposta/provocação",
  "form": "Nome da forma despertada",
  "boss": false,
  "speed": 1.0,
  "stats": {},
  "art": { "type": "rig", "style": {}, "poses": {} },
  "palettes": [],
  "anims": {},
  "moves": []
}
```

| Campo | Obrigatório | Descrição |
|---|---|---|
| `format` | sim | Sempre `umf/1` |
| `id`, `name`, `series`, `author`, `color` | sim | Identidade; `color` é a cor do cartão na seleção |
| `boss` | não | `true` esconde da seleção e coloca como chefe final do Arcade |
| `speed` | não | Multiplicador de velocidade das animações (>1 = golpes mais rápidos) |
| `stats` | não | Atributos, tabela abaixo |
| `art` | sim | Arte, ver [Rig](Rig.md) |
| `palettes` | não | Cores alternativas; cada item sobrescreve chaves de `art.style.colors` |
| `anims` | sim | Animações por nome |
| `moves` | sim | Golpes |

## `stats`

| Campo | Padrão de exemplo | Significado |
|---|---|---|
| `life` | `1000` | Vida |
| `power` | `3000` | Barra máxima de super (1000 por nível) |
| `attack` / `defense` | `100` | Multiplicadores em % |
| `walk` | `[2.4, 2.2]` | Velocidade andando para frente / para trás |
| `run` | `[5.2, 4.6]` | Corrida para frente / recuo |
| `jump` | `[-8.4, -2.5, 0, 2.6]` | Velocidade vertical, horizontal para trás, neutra e para frente |
| `gravity` | — | Aceleração vertical |
| `airJumps` | — | Pulos extras no ar |
| `height` | `100` | Altura do corpo |
| `width` | `[16, 15]` | Largura frente / trás para empurrão |

## `anims`

Cada animação é uma lista de quadros:

```json
"jab": { "frames": [
  { "pose": "jab0", "ticks": 3 },
  { "pose": "jab1", "ticks": 3, "hit": { "part": "handR", "radius": 10 } },
  { "pose": "jab0", "ticks": 5 }
], "loop": 0 }
```

| Campo do quadro | Descrição |
|---|---|
| `pose` | Nome de uma pose em `art.poses` ou a pose inline (objeto) |
| `ticks` | Duração em ticks (60 por segundo); `-1` = infinito |
| `hit` | Hitbox ativa: `part` (membro ou arma) + `radius`, ou `box` `[x1, y1, x2, y2]` |
| `hurt` | `"none"` remove a hurtbox (invencível no quadro) |
| `effect` | Desenha um efeito em vez do corpo (projéteis, faíscas), ver [Rig](Rig.md#efeitos) |
| `offset` | Deslocamento `[x, y]` do desenho |
| `loop` | Índice do quadro onde o loop recomeça |

### Nomes padrão

Animações com estes nomes viram as ações reservadas do M.U.G.E.N, então os estados comuns funcionam sem código.

| Nome | Ação | Nome | Ação |
|---|---|---|---|
| `idle` | 0 | `guardHit` | 150 |
| `turn` | 5 | `crouchGuardHit` | 152 |
| `crouchTurn` | 6 | `airGuardHit` | 154 |
| `crouchStart` / `crouch` / `crouchEnd` | 10 / 11 / 12 | `lose` / `draw` / `win` | 170 / 175 / 181 |
| `walk` / `walkBack` | 20 / 21 | `intro` | 190 |
| `jumpStart` / `jumpUp` / `jumpFwd` / `jumpBack` / `jumpDown` | 40–44 | `taunt` / `powerUp` | 195 / 196 |
| `land` | 47 | `defeat1`…`defeat4` | 197–200 |
| `run` / `hop` | 100 / 105 | `hitLight` / `hitMedium` / `hitHeavy` | 5000–5002 |
| `guardStart` / `crouchGuardStart` / `airGuardStart` | 120–122 | `hitLowLight` / `hitLowMedium` / `hitLowHeavy` | 5010–5012 |
| `guard` / `crouchGuard` / `airGuard` | 130–132 | `hitCrouch` / `hitAir` | 5020 / 5030 |
| `guardEnd` | 140 | `airRecover` / `fall` / `trip` | 5040 / 5050 / 5070 |
| | | `bounce` / `bounceAir` / `lieDown` / `getUp` / `dead` | 5100 / 5101 / 5110 / 5120 / 5150 |

`taunt`, `powerUp` e `defeat*` têm versão padrão se você não definir. Outros nomes recebem números livres na ordem do arquivo.

## `moves`

```json
{ "id": "gale", "name": "Onda de Vento", "input": "~D, DF, F, x", "simple": "sp1",
  "kind": "special", "anim": "gale", "invuln": [1, 3],
  "moves": [{ "at": 2, "vel": [2, -9] }],
  "hits": [{ "at": 2, "damage": 90, "level": "heavy", "launch": true }],
  "projectile": { "at": 3, "anim": "galeOrb", "hitAnim": "galeHit", "speed": [5, 0], "offset": [40, -60], "damage": 70 } }
```

| Campo | Valores | Efeito |
|---|---|---|
| `id`, `name` | texto | Identificador e nome na lista de golpes |
| `input` | sintaxe de comando, abaixo | Comando |
| `simple` | `sp1` `sp2` `sp3` `super` | Atalho de um botão (teclas 1/2/3, toque) |
| `kind` | `normal` `special` `super` | Prioridade e cancelamento: normal → especial → super em acerto ou defesa |
| `when` | `stand` (padrão) `crouch` `air` | Estado de onde o golpe sai |
| `anim` | nome em `anims` | Animação do golpe |
| `hits` | lista | Um HitDef por item |
| `moves` | `[{ "at": quadro, "vel": [x, y] }]` | `VelSet` no quadro; use `null` para manter um eixo |
| `projectile` | objeto ou lista | Projéteis |
| `cost` | número | Barra consumida; `super` usa 1000 por padrão |
| `invuln` | `[início, fim]` | Quadros invencíveis |
| `teleport` | quadro | Reaparece atrás do oponente |
| `fx` | `aura` | Efeito visual extra |
| `airPhysics` | `true` | Mantém física aérea |

### `hits[]`

| Campo | Padrão | Descrição |
|---|---|---|
| `at` | — | Quadro (1 = primeiro) em que o HitDef é ativado |
| `damage` | — | Dano |
| `guardDamage` | `damage / 8` | Dano na defesa |
| `level` | `medium` | `light` (hitstop 9, hitstun 12), `medium` (11, 16), `heavy` (13, 20) |
| `guard` | `mid` | `high`, `low`, `mid` ou `unblockable` |
| `knockback` / `airKnockback` | pelo nível | Velocidade `[x, y]` aplicada ao oponente |
| `launch` | `false` | Derruba |
| `hitstop`, `hitstun`, `spark`, `shake` | pelo nível | Ajuste fino |

### `projectile`

| Campo | Descrição |
|---|---|
| `at` | Quadro em que sai |
| `anim` / `hitAnim` | Animação em voo / ao acertar |
| `speed` / `accel` | Velocidade e aceleração `[x, y]` |
| `offset` | Posição de saída `[x, y]` relativa ao lutador |
| `damage`, `level`, `launch`, `knockback` | Como em `hits` |
| `hits` | Número de acertos |
| `removeTime` | Ticks até sumir |
| `priority` | Prioridade contra outros projéteis |

### Sintaxe de comando

| Símbolo | Significado | Exemplo |
|---|---|---|
| `F B U D` / `DF DB UF UB` | Direções (relativas ao lado) | `D, DF, F, x` |
| `x y z a b c` | Botões (`s` = start) | `x` |
| `,` | Em sequência | `F, F` |
| `+` | Ao mesmo tempo | `x+y` |
| `~` | Soltar | `~D, DF, F, x` |
| `/` | Segurar | `/B, F, x` |
| `$` | Direção inclusiva (`$D` aceita DB, D, DF) | `$D, x` |
| `>` | Sem nenhuma outra entrada entre os passos | `F, >F` |

## Validar

```bash
cargo test --workspace --release
cargo run --release -p ume-engine --example rig_sheet
```

`rig_sheet` gera uma folha com todas as poses para revisar a arte sem abrir o jogo.
