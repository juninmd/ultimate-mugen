# Biblioteca de habilidades

Blocos reutilizáveis para lutadores UMF. Um lutador convertido ou escrito à mão deve citar uma habilidade e ajustá-la, não reescrever os controladores. Os blocos são campos de dados do motor mais um kit embutido (`abilities`) com golpes prontos.

## Mapa de lacunas: o que o pacote MUGEN faz e onde o UMF cobre

| Padrão no pacote | Bloco UMF | Situação |
|---|---|---|
| Helper de bola de fogo, `Projectile` | `projectile` | existente |
| Rajada de ki (vários `Projectile`, um por quadro) | `projectile.repeat` | novo |
| Teleporte atrás ou até o rival (`PosAdd p2dist`, `Turn`) | `blink` (`behind`, `front`, `above`, `back`) e o antigo `teleport` | novo |
| Flash de carga, explosão de aura, `Explod` de animação | `vfx` | novo |
| `EnvShake` fora de um golpe | `quake` | novo |
| Raio com vários golpes (`projhits`) | `projectile.hits` | existente |
| Voo, clone, forma (`var`) | `abilities.fly`, `helpers.clones`, `abilities.forms` | existente |
| Rush com vários `HitDef` | `hits[]` e `moves[].vel` | existente |
| Helper com IA própria, `Statedef -2/-3`, Lua/ZSS | `scripts`, ou ficar no formato legado | não migra |

## Campos do golpe

```json
{ "id": "salvo", "kind": "special", "anim": "arcbolt", "cost": 250,
  "blink": { "at": 2, "to": "behind", "gap": 36 },
  "vfx": [{ "at": 1, "anim": "boltHit", "offset": [0, -50], "time": 12, "follow": true, "scale": 1.5 }],
  "quake": [{ "at": 3, "time": 12, "amp": 2 }],
  "projectile": { "at": 3, "anim": "boltFx", "speed": [6, 0], "offset": [44, -78], "damage": 22,
                  "repeat": { "count": 5, "every": 1, "offsetStep": [0, 8], "speedStep": [0.4, 0], "damageStep": 0 } } }
```

| Campo | Significado |
|---|---|
| `blink` (objeto ou lista) | quadro `at`, `to` = `behind` (vira, padrão), `front`, `above`, `back`; `gap` distância (padrão 40); `y` deslocamento vertical |
| `vfx[]` | Um `Explod` de uma animação do lutador no quadro `at`: `offset` (padrão `[0, -50]`), `time` em ticks (padrão: até o fim da animação), `follow` prende ao lutador, `scale`, `behind` desenha sob o lutador |
| `quake[]` | `EnvShake` no quadro `at`: `time` (8), `amp` (4) |
| `projectile.repeat` | Abre um projétil em `count` (máx. 24): o k-ésimo sai no quadro `at + k * every` (padrão 2), deslocado por `k * offsetStep`, `k * speedStep` e `k * damageStep`. Uma rajada maior que a animação dispara o resto no último quadro |

`vfx` reutiliza animações que o lutador já tem (quadros PNG). O motor nunca desenha um quadro por código.

## Kit embutido `abilities`

Um kit de biblioteca: `"extends": ["abilities"]` não adiciona nada sozinho; só entram os golpes que você citar (por id ou com `use`). Golpes: `ki_blast`, `ki_barrage`, `beam`, `instant_transmission`, `blink_strike`, `power_surge`. Cada um traz números razoáveis; mude só o que difere. `projectile` e `blink` se mesclam campo a campo (veja o exemplo em [Ability library](../Ability-Library.md)).

Sempre troque `anim` (e `projectile.anim`) pelas animações do próprio lutador. Ferramentas que leem o arquivo do elenco sem `parse_char` (alguns testes) não entendem `extends`; os lutadores do elenco escrevem os blocos por extenso, e os kits servem a pacotes e mods.

## Conversor

`tools/mugen2umf` converte `Explod` em `explods` e `EnvShake` em `shake` (com `blink`, `vfx` e `quake` disponíveis para escrita manual) e compara nomes de `command` sem diferenciar caixa, como o MUGEN faz (`SPECIAL 1` no `.cmd`, `"special 1"` no gatilho). Antes da correção, um pacote assim ficava só com os normais: o Goku SS4 passou de 5 para 15 golpes.

## Regras práticas

- Repetir os mesmos controladores em dois golpes indica um bloco faltando: crie um campo ou um golpe de kit.
- Um bloco novo precisa de teste unitário da expansão e de um teste do motor que o execute (`crates/engine/tests/native_ability_blocks.rs`).
- Confira cada `at` contra a duração da animação; o compilador não estica animação.
