---
name: mugen-to-ume-migrator
description: >-
  Migra um personagem M.U.G.E.N (.def/.sff/.air/.cmd/.cns/.snd) para um
  personagem nativo Ultimate MUGEN (UMF JSON em assets/chars). Use ao converter
  golpes, HitDefs, projéteis, comandos, hitboxes e sprites de um lutador MUGEN.
---

# Migração MUGEN → Ultimate MUGEN (UMF)

O motor já roda personagens MUGEN legados direto da pasta ou do zip. Migre para UMF só quando quiser: um único JSON, habilidades declarativas (`fly`, `teleport`, `superSpeed`, `forms`, `passives`) e carga mais leve. Não migre arte de terceiros sem licença; sprites e stages do projeto são recriados pelo usuário, então a migração cobre dados (golpes, física, comandos), não arte.

## Conversor automático (recomendado)

`python3 .agents/skills/mugen-to-ume-migrator/scripts/mugen2umf.py <pasta-mugen> <saida> --id <id> --name "<Nome>" [--extra extra.json]` gera um pacote UMF (`<id>.json` + `sprites/`) fiel ao original. Depende de `numpy` e `Pillow`.

- **Sprites**: cada `(grupo, índice)` usado vira `sprites/s{g}_{i}.png` sem reamostragem; `meta.json` guarda `axisX/axisY` do SFF e `refHeight` = altura do idle.
- **Escala 1:1**: `stats.height` = `refHeight` (escala = 100/refHeight × height/100 = 1). Nunca use 100 nem outro valor, senão o lutador fica maior e borrado.
- **Caixas**: `Clsn1` vira `hit.box` (união das caixas), `Clsn2` vira `hurt` por quadro, e `art.style.boxes = "exact"` desliga o recorte automático pela máscara do sprite.
- **Golpes**: lidos do `Statedef -1` (`ChangeState` + `command`). Um grupo `triggerN` com `ctrl = 0` ou que só serve a outro estado é ignorado, e comandos só de segurar (`/x`) também. `statetype` define `when`, e `power >= N` define `cost`.
- **Cadeias**: `ChangeState` sem `command` (ex.: 4010 → 4011 → 4012) é seguido até 4 estados e vira uma anim `seq<estado>`, com os `at` deslocados.
- **HitDef/Projectile/VelSet/PosAdd p2dist/PlaySnd** viram `hits`, `projectile`, `moves`, `teleport` e `sounds` no quadro certo (`AnimElem`, ou `Time` convertido em quadro).
- **Sons**: o `.snd` é extraído para `sprites/snd_{g}_{i}.wav` e listado em `meta.json.sounds`; o motor carrega esses arquivos e toca `moves[].sounds` (`{at, snd:[g,i], fight?}`) no quadro indicado.
- **Super**: só com `SuperPause`, `poweradd <= -1000` ou custo ≥ 1000. Número de estado ≥ 3000 não basta, porque teleportes vivem lá.
- **Efeitos**: `Explod` de animação vira `moves[].vfx` (até 3) e `EnvShake` vira `moves[].quake` (até 2). `command` é comparado sem diferenciar caixa (`SPECIAL 1` no `.cmd` e `"special 1"` no gatilho); sem isso o pacote perde quase todos os especiais.
- **`--extra`**: um JSON mesclado por cima (moves, anims, abilities, helpers) para o que helpers e variáveis não expressam.

Pegadinhas: o nome do `.act` pode ter outra caixa (a busca não diferencia maiúsculas); `[State N]` com número diferente do `Statedef` pertence ao `Statedef` corrente; a paleta 1 só vale para sprites `same_pal`.

## Fluxo

1. **Auditar o original**: `cargo run --release -p ume-charforge -- check <pasta> --strict` lista controladores e triggers não suportados.
2. **Inventário** do `.cmd`/`.cns`: comandos, `Statedef` de ataque (200-299, 400-499, 600-699, 1000+, 3000+), helpers de projétil, estados de arremesso, transformações.
3. **Copiar um nativo** como base (`cp assets/chars/kaito.json assets/chars/<id>.json`) e trocar `id`, `name`, `series`, `author`, `color`.
4. **Converter** pela tabela abaixo.
5. **Validar**: `cargo test --release -p ume-engine --test native_abilities_and_forms` e os testes `native_*` do roster; abrir em Training com `F2`.
6. Registrar o `id` em `assets/chars/index.json` e documentar em `docs/wiki/`.

## Tabela de mapeamento

| MUGEN | UMF |
|---|---|
| `[Data] life/power/attack/defence` | `stats.life/power/attack/defense` |
| `[Velocity] walk.fwd/back`, `run.fwd`, `jump.*` | `stats.walk`, `stats.run`, `stats.jump` (dividir por 4 se vier em 1280x720) |
| `[Movement] airjump.num`, `yaccel` | `stats.airJumps`, `stats.gravity` |
| `[Size] height`, `ground.front/back` | `stats.height`, `stats.width` |
| `[Command] command = ~D, DF, F, x` | `moves[].input` (mesma sintaxe) |
| Ação `.air` | `anims.<nome>` (frames `pose`, `ticks`, `hit`, `hurt`) |
| Nomes de ação 0, 20, 40... | nomes padrão (`idle`, `walk`, `jumpUp`...) |
| `Clsn1` ativo | quadro com `hit` (`part`+`radius` ou `box`) |
| `HitDef damage` | `hits[].damage` (+ `guardDamage`) |
| `animtype Light/Medium/Heavy` | `hits[].level` |
| `guardflag`, `guard` | `hits[].guard` (`high`/`low`/`mid`/`unblockable`) |
| `ground.velocity` / `air.velocity` | `knockback` / `airKnockback` |
| `fall = 1` | `launch: true` |
| `pausetime`, `ground.hittime` | `hitstop`, `hitstun` |
| `sparkno`, `hitsound` | `spark`, `hitSound` |
| `VelSet` num frame | `moves[{at, vel}]` |
| `NotHitBy` / invulnerabilidade | `invuln: [from, to]` |
| `Helper` com `HitDef` (bola de fogo) | `projectile` (`speed`, `offset`, `hits`, `removeTime`) |
| `Projectile` | `projectile` |
| `TargetState` + `TargetBind` | `throw` (`bind`, `launch`, `at`) |
| `ReversalDef` | `parry.window` |
| `HitOverride` / super armor | `armor.hits` |
| `SuperPause`, power `poweradd` | `kind: "super"` + `cost` |
| `Pause`/time stop | `timestop` |
| `Explod` de aura/flash, `EnvShake` | `moves[].vfx`, `moves[].quake` |
| Vários `Projectile` em sequência (rajada de ki) | `projectile.repeat {count, every, offsetStep, speedStep}` |
| Voo `physics = N` | `abilities.fly` ou `flight: true` |
| Teleporte (`PosSet` atrás do inimigo) | `abilities.teleport`, `moves[].teleport` ou `moves[].blink {to, gap}` |
| `ChangeState` de transformação | `abilities.forms[]` |
| Raiva, regeneração, buffs por var | `passives` (`rage`, `enrageTime`, `stacks`, `lifeRegen`, `meterRegen`) |
| `life <= lifemax/2`, `p2life <= ...` no gatilho do comando | `requires.life` / `requires.enemyLife` (`min`/`max` em fração da vida) |
| Cancels com `var(1)`/`movecontact` | `cancels: [ids]` |

## O que não migra

Lua/ZSS arbitrário, helpers com lógica própria, `Statedef -2/-3` customizados e estados de IA: recrie como `passives`, `forms` ou deixe o personagem em formato legado. Nunca copie nomes de arquivo de terceiros ou arte; crie poses novas ou reuse as `anims` existentes do nativo.

## Regras do repositório

Sem comentários em código; arquivos de código até 250 linhas; PR com título `type: description`; teste de fumaça no navegador (wasm) antes de entregar.

## Biblioteca de habilidades

Antes de copiar controladores para um lutador, veja `docs/wiki/pt-BR/Biblioteca-de-Habilidades.md`: blocos (`blink`, `vfx`, `quake`, `projectile.repeat`) e o kit `abilities` (`"extends": ["abilities"]` + `"use": "ki_barrage"`) deixam o JSON do lutador curto. Padrão repetido em duas conversões vira bloco novo no motor, com teste.

## Aprender antes de migrar

Para entender como um pacote foi feito (comandos, hitdefs, helpers, projéteis, formas), rode a skill `mugen-dissector` primeiro; a tabela de padrões e os números de referência estão em `docs/wiki/pt-BR/Dissecacao-MUGEN.md`.
