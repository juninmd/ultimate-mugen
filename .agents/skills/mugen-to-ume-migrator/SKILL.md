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

Todo o código de migração é Rust e vive em uma pasta só, `tools/mugen2umf/` (crate `mugen2umf`), usado como CLI ou servidor MCP (`.mcp.json`):

```
cargo run --release -p mugen2umf -- convert <pasta-ou-zip> <saida> <id> [--name "<Nome>"] [--extra extra.json]
cargo run --release -p mugen2umf -- inspect <pasta-ou-zip>        # arquivos, SFF, comandos, estados, controladores, rotas, suporte do motor
cargo run --release -p mugen2umf -- hitdefs <pasta-ou-zip>        # todas as HitDefs por estado
cargo run --release -p mugen2umf -- parity <pasta-ou-zip> <saida> # golpe a golpe, tick a tick
cargo run --release -p mugen2umf -- validate <saida>              # carrega e compila como o jogo
cargo run --release -p mugen2umf -- profile_facts <pasta-ou-zip>  # fatos do pacote + campos UMF a preencher
cargo run --release -p mugen2umf -- set_profile <saida> '{"series":"...","profile":{...}}'
cargo run --release -p mugen2umf -- mcp                           # as mesmas ferramentas via MCP (stdio)
```

Testes: `cargo test --release -p mugen2umf` (converte o Kung Fu Man de `assets/mugen/elecbyte.zip` e exige paridade total).

### Modo exato: comportamento 100% idêntico

A migração não reinterpreta mais os estados como golpes UMF (era isso que perdia comportamento: Black Widow e Goku SSJ4). O bundle guarda o personagem original e o motor carrega pelo mesmo código de um personagem M.U.G.E.N (`ume_engine::loader::assemble_mugen_char`):

- `<id>.json` → bloco `mugen`: `[Info]` original, `cns`, `st*`, `stcommon` e `cmd` verbatim (texto), e `anims` = cada action do AIR em JSON (sprite, offset, ticks, flip, blend/alpha, escala, ângulo, Clsn1/Clsn2 completos, loopstart).
- `sprites/` → cada sprite do SFF como PNG indexado (índices e paleta exatos, tRNS completo; true color vira RGBA), as paletas do SFF (`sffpal_<k>.bin`), os `.act` de `pal1..pal12` (troca de paleta igual ao MUGEN), todos os sons (`snd_g_i.wav`) e `meta.json` (eixos, tamanhos, paleta de cada sprite, ordem do SFF).
- `art.type = "mugen"`, `anims: {}` e `moves: []`: os golpes vêm do próprio CNS.
- Saídas extras: `parity.md` (relatório), `routes.json` (comando → estado) e `report.md` (inspeção + diagnóstico do motor).

`parity` joga cada comando do `Statedef -1` pelo input real no original e no UMF (150 ticks, mesma luta) e compara, a cada tick, estado, animação, quadro, posição, velocidade, controle, vida, poder, helpers e sons; e compara os dados (estados, comandos, constantes, animações, sprites, paletas, sons). Meta: **IDENTICAL**. Diferença aqui é bug do conversor ou do formato, nunca "aceitável".

O que o motor ainda não suporta do MUGEN aparece no `report.md` (diagnóstico) e afeta original e UMF igualmente; corrija no motor, não no conversor.

### Melhorias UMF por cima (`--extra`)

- Metadados e campos exclusivos do UMF: `name`, `series`, `author`, `description`, `color`, `intro`, `taunt`, `winQuote`, `rivals`, `profile` (ver abaixo).
- `moves: [{id, name}]` sem `anim` só renomeia o golpe na lista (`mugen.names`, chave = nome do comando).
- `moves` UMF completos (com `anim`, `input`, `kind`, `hits`, `projectile`, `throw`...) viram golpes nativos por cima do exato: `anim` é o número da action M.U.G.E.N (`"3000"`), estados num bloco de 100000 acima do maior estado original, rotas antes do `Statedef -1` original. O `parity.md` lista em "Added by UMF" se o input real alcança e quanto dano dá.
- `mugenRoutes: [{state, id, input, name?, when?, cost?}]` destrava estados que o `.cmd` deixou sem comando (ex.: hyper S.H.I.E.L.D. da Black Widow) via `scripts`.
- `scripts.cns`/`scripts.zss`/`scripts.commands`: golpes novos escritos em CNS/ZSS, com `@anim(<número>)`, somados depois do original (o `Statedef -1` novo roda depois do original).

### Campos exclusivos do UMF preenchidos pela skill

1. `profile_facts <pacote>` devolve o `[Info]` do `.def`, readmes/notas do pacote e a lista de campos.
2. A skill (o modelo) escreve `series`, `description`, falas e `profile` (`country` ISO, `nationality`, `origin`, `bio`, `title`, `fightingStyle`, `affiliation` em `en`, `pt` e `es`, `element`, `alignment`, `tags`) fiel ao personagem.
3. `set_profile <saida> '<json>'` valida (serde do jogo) e grava. Nada é inventado pelo código: o código junta fatos, o modelo redige.

## Enriquecimento UME obrigatório (identidade, país, interações e storyboard)

Ao migrar um lutador M.U.G.E.N para o padrão UMF 2.0, **sempre preencha todos os blocos exclusivos do UME** (via `--extra` ou editando o JSON final) mantendo **100% de fidelidade ao personagem original**:

1. **Identidade & Falas Fiéis (`<id>.json`)**:
   - `series` (obra/franquia original), `author` (criador original + crédito de conversão), `color` (cor assinatura em hex), `form` (nome da forma/despertar) e `description` (resumo curto fiel).
   - `intro`: frase de entrada icônica antes da luta.
   - `taunt`: provocação autêntica na personalidade do personagem.
   - `winQuote`: frase de vitória clássica.
   - `rivals`: interações pré-luta específicas (`{ "<rival_id>": { "intro": "...", "taunt": "...", "attack": 1.06 } }`).
2. **País de Origem & Perfil Completo (`profile` no `<id>.json` + `assets/char_profiles.json` se nativo fixo)** — preencher em `en`, `pt` e `es`:
   - `country`: código ISO minúsculo usado pela bandeira no seletor (ex.: `"jp"`, `"br"`, `"cn"`, `"us"`, `"no"`, `"de"`, `"ph"`, `"mx"`).
   - `nationality`: nacionalidade / raça (`{"en": "...", "pt": "...", "es": "..."}`).
   - `origin`: cidade, planeta ou reino de origem (`{"en": "...", "pt": "...", "es": "..."}`).
   - `title`: título / epíteto (ex.: `"Super Saiyan 4"`).
   - `fightingStyle`: estilo de luta (ex.: `"Turtle School Martial Arts"`).
   - `affiliation`: equipe / facção / clã (ex.: `"Z Fighters"`).
   - `bio`: mini biografia fiel ao cânone (`{"en": "...", "pt": "...", "es": "..."}`).
   - `element`: elemento principal (`"ki"`, `"fire"`, `"wind"`, `"lightning"`, `"ice"`, `"shadow"`, `"earth"`, `"light"`, `"water"`).
   - `weakness`: elemento fraqueza (ex.: `"dark"`, `"ice"`, `"earth"`).
   - `alignment`: alinhamento moral — `"hero"`, `"villain"`, `"anti-hero"` ou `"neutral"`.
   - `tags`: arquétipos de gameplay (ex.: `["rushdown", "beam", "teleport", "aerial"]`).
3. **Interações com Outros Chars e Cenários (`assets/interactions.toml`)**:
   - **`[[matchup]]`**: diálogos/banners contra rivais específicos (`fighters = ["id_a", "id_b"]`, `style = "ribbon" | "speech"`, `sprite`, `sfx`, `color`, `title_pt/en/es`, `sub_pt/en/es`).
   - **`[[stage_interaction]]`**: reações a cenários icônicos (`fighter`, `stage_contains = ["vulc", "volcano", ...]`, `speaker`, `style`, `sprite`, `sfx`, `color`, `title_pt/en/es`, `sub_pt/en/es`).
   - **`[[fighter_trigger]]`**: falas de provocação (`trigger = "taunt"`, `per_round = true`) e virada com vida baixa (`trigger = "low_hp"`, `threshold = 0.30`).
4. **Storyboard & Epílogo (`assets/story/epilogues.json`)**:
   - Adicione a entrada `"<id>"` com `scene` (stage de fundo), `title`, `boss` (fala final do chefe), `reply` (resposta icônica do lutador) e `pages[]` (quadros narrativos do storyboard pós-vitória em `en`, `pt`, `es`). Valide com `UME_TOUR=epilogue` ou `?e2e_epilogue=1`.
   - Se o pacote M.U.G.E.N trouxer `.def` de storyboard/ending ou texto de história (`MugeNow.txt` / intros CNS), adapte fielmente os diálogos para `epilogues.json` e `intro`/`winQuote`.

Pegadinhas aprendidas (estudo de caso Goku SSJ4): nunca siga `ChangeState` para o estado de voo (`3001`) em cadeias de ataque; remova `loop` e ticks `-1` de `intro`, `taunt` e ataques para `AnimTime = 0` disparar; preserve múltiplos `PosAdd` em `teleport[]` (ex.: subida e retorno da Genki Dama) e garanta saída aérea (`value = 50`) quando `Pos Y < 0`; em `ChangeAnim` de voo, cheque `command = "holdup"` (regex) e não substring `"holdup"` (senão casa `!= "holdup"`); o nome do `.act` pode ter outra caixa; `[State N]` com número diferente do `Statedef` pertence ao `Statedef` corrente; a paleta 1 só vale para sprites `same_pal`.

## Fluxo

1. **Auditar o original**: `cargo run --release -p ume-charforge -- check <pasta> --strict` lista controladores e triggers não suportados.
2. **Inventário** do `.cmd`/`.cns`: comandos, `Statedef` de ataque (200-299, 400-499, 600-699, 1000+, 3000+), helpers de projétil, estados de arremesso, transformações.
3. **Copiar um nativo** como base (`cp assets/chars/kaito.json assets/chars/<id>.json`) e trocar `id`, `name`, `series`, `author`, `color`.
4. **Converter** pela tabela abaixo.
5. **Validar**: `node scripts/umf-schema.mjs assets/chars/<id>.json`, `cargo test --release -p ume-engine --test native_abilities_and_forms` e os testes `native_*` do roster; abrir em Training com `F2`.
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
| `Explod` de aura/flash, `EnvShake` | `moves[].explods`, `moves[].shake` |
| Vários `Projectile` em sequência (rajada de ki) | `projectile.repeat {count, every, offsetStep, speedStep}` |
| Voo `physics = N` | `abilities.fly` ou `flight: true` |
| Teleporte (`PosSet` atrás do inimigo) | `abilities.teleport`, `moves[].teleport` ou `moves[].blink {to, gap}` |
| `ChangeState` de transformação | `abilities.forms[]` |
| Raiva, regeneração, buffs por var | `passives` (`rage`, `enrageTime`, `stacks`, `lifeRegen`, `meterRegen`) |
| `life <= lifemax/2`, `p2life <= ...` no gatilho do comando | `requires.life` / `requires.enemyLife` (`min`/`max` em fração da vida) |
| Cancels com `var(1)`/`movecontact` | `cancels: [ids]` |

## O que não migra

No modo exato tudo migra (estados, helpers, `Statedef -2/-3`, ZSS, IA, paletas, sons). O limite é o motor: o que ele não suporta do MUGEN aparece no `report.md` e vale igual para o original. A tabela acima serve só para recriar um lutador à mão como UMF nativo (`moves`/`anims`).

## Regras do repositório

Sem comentários em código; arquivos de código até 250 linhas; PR com título `type: description`; teste de fumaça no navegador (wasm) antes de entregar.

## Biblioteca de habilidades

Antes de copiar controladores para um lutador, veja `docs/wiki/pt-BR/Biblioteca-de-Habilidades.md`: blocos (`blink`, `vfx`, `quake`, `projectile.repeat`) e o kit `abilities` (`"extends": ["abilities"]` + `"use": "ki_barrage"`) deixam o JSON do lutador curto. Padrão repetido em duas conversões vira bloco novo no motor, com teste.

## Aprender antes de migrar

Para entender como um pacote foi feito (comandos, hitdefs, helpers, projéteis, formas), rode a skill `mugen-dissector` primeiro; a tabela de padrões e os números de referência estão em `docs/wiki/pt-BR/Dissecacao-MUGEN.md`.
