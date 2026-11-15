---
name: mugen-to-ume-migrator
description: >-
  Migra um personagem M.U.G.E.N (.def/.sff/.air/.cmd/.cns/.snd) para um
  personagem nativo Ultimate MUGEN (UMF JSON em assets/chars). Use ao converter
  golpes, HitDefs, projéteis, comandos, hitboxes e sprites de um lutador MUGEN.
---

# Migração MUGEN → Ultimate MUGEN (UMF)

O motor já roda personagens MUGEN legados direto da pasta ou do zip. Migre para UMF só quando quiser: um único JSON, habilidades declarativas (`fly`, `teleport`, `superSpeed`, `forms`, `passives`) e carga mais leve. Não migre arte de terceiros sem licença; sprites e stages do projeto são recriados pelo usuário, então a migração cobre dados (golpes, física, comandos), não arte.

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
| Voo `physics = N` | `abilities.fly` ou `flight: true` |
| Teleporte (`PosSet` atrás do inimigo) | `abilities.teleport` ou `moves[].teleport` |
| `ChangeState` de transformação | `abilities.forms[]` |
| Raiva, regeneração, buffs por var | `passives` (`rage`, `enrageTime`, `stacks`, `lifeRegen`, `meterRegen`) |
| `life <= lifemax/2`, `p2life <= ...` no gatilho do comando | `requires.life` / `requires.enemyLife` (`min`/`max` em fração da vida) |
| Cancels com `var(1)`/`movecontact` | `cancels: [ids]` |

## O que não migra

Lua/ZSS arbitrário, helpers com lógica própria, `Statedef -2/-3` customizados e estados de IA: recrie como `passives`, `forms` ou deixe o personagem em formato legado. Nunca copie nomes de arquivo de terceiros ou arte; crie poses novas ou reuse as `anims` existentes do nativo.

## Regras do repositório

Sem comentários em código; arquivos de código até 250 linhas; PR com título `type: description`; teste de fumaça no navegador (wasm) antes de entregar.
