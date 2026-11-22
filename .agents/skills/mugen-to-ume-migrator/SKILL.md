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

`python3 .agents/skills/mugen-to-ume-migrator/scripts/mugen2umf.py <pasta-mugen> <saida> --id <id> --name "<Nome>" [--extra extra.json] [--report]` gera um pacote UMF (`<id>.json` + `sprites/`) fiel ao original. Depende apenas de `Pillow` (sem `numpy`).

- **Sprites**: todo `(grupo, índice)` do SFF vira `sprites/s{g}_{i}.png` sem reamostragem; `meta.json` guarda `axisX/axisY` do SFF e `refHeight` = altura do idle.
- **Escala 1:1**: `stats.height` = `refHeight` (escala = 100/refHeight × height/100 = 1). Nunca use 100 nem outro valor, senão o lutador fica maior e borrado.
- **Caixas**: `Clsn1` vira `hit.box` (união das caixas), `Clsn2` vira `hurt` por quadro, e `art.style.boxes = "exact"` desliga o recorte automático pela máscara do sprite.
- **Golpes e cancelamentos**: lidos do `Statedef -1` (`ChangeState` + `command`). Grupos `stateno = X` + `movecontact`/`movehit` viram `cancels: [...]`. Um grupo `triggerN` com `ctrl = 0` ou que só serve a outro estado é ignorado, e comandos só de segurar (`/x`) também. `statetype` define `when`, e `power >= N` define `cost`. `command` é comparado sem diferenciar caixa (`SPECIAL 1` no `.cmd` e `"special 1"` no gatilho); sem isso o pacote perde quase todos os especiais.
- **Habilidades automáticas (`mugen_abilities.py`)**: detecta estados de voo (direcionais `holdfwd/back/up/down` → `abilities.fly` com `upAnim/downAnim/fwdAnim/backAnim`), carga de ki (`PowerAdd > 0` com loop → `abilities.charge` com `auraColor/shake/sound`) e teleporte para trás do oponente só quando usa `p2dist` (`abilities.teleport`, aceita input de movimento como `~B, D, B, x` com `at` sincronizado no quadro do `PosAdd` e encerramento em `AnimTime = 0`), removendo-os da lista de ataques para evitar travamentos (`ctrl = 0`).
- **Cadeias e hit-confirm (`followUp`)**: `ChangeState` incondicional é seguido até 12 estados (`seq<estado>`) sem entrar em estados consumidos de voo/carga. Quando há transição condicional (`MoveHit = 1` / `MoveContact = 1`), o conversor divide em golpe de follow-up e emite `followUp: { to, on: "hit" | "contact" | "hitNow" }`, evitando cinemáticas no whiff. Os `at` são deslocados.
- **HitDef/Projectile/VelSet/PosAdd p2dist/PlaySnd** viram `hits`, `projectile`, `moves`, `teleport` e `sounds` no quadro certo (`AnimElem`, ou `Time` convertido em quadro).
- **FX, cinemática e teleportes multi-etapa**: `Explod` → `explods[]` (incluindo anims e sprites no pacote), `AfterImage` → `afterimage`, `EnvShake` → `shake`, `EnvColor` → `flash`, `SuperPause` → `superPause`, `NotHitBy` → `invuln`, e todos os `PosAdd` deduplicados da sequência → `teleport` único ou lista `[step1, step2]` (ex.: Super Genki Dama sobe `[-200, -100]`, lança e retorna `[125, 100]`). Golpes com teleporte que terminam com `Pos Y < 0` saem automaticamente para o estado aéreo `50` (`ctrl = 1`) em vez de flutuar travados no estado `0`.
- **Super**: só com `SuperPause`, `poweradd <= -1000` ou custo ≥ 1000. Número de estado ≥ 3000 não basta, porque teleportes vivem lá.
- **Feixes de Ki (`Projectile`)**: preserva `projhits` (`hits`), `projmisstime` (`missTime`), `hitflag`, `guardDamage`, `pausetime`, `hitSound`, `accel`, `knockback` e `launch`.
- **Agarrões e Wall Bounce**: `TargetBind` + `TargetLifeAdd` + `ChangeAnim2` viram `throw`; estados de P2 com `BackEdgeBodyDist` ativam `wallBounce: true` no `HitSpec`.
- **Sons**: o `.snd` inteiro é extraído para `sprites/snd_{g}_{i}.wav` e listado em `meta.json.sounds`; o motor carrega esses arquivos e toca `moves[].sounds` (`{at, snd:[g,i], fight?}`) no quadro indicado.
- **Intro, provocação e estados especiais (`mugen_scripts.py`)**: estados 170-199 (intro, vitória, provocação) e o que eles chamam viram `scripts.cns` verbatim com `@anim(nome)`; o motor entra no `Statedef 190` próprio no início do round, então texto/balão de entrada e sons saem iguais. Comandos do `Statedef -1` usados nesses estados viram `umf_<nome>`; provocação no Start sai para `x+y` (Taunt), porque Start/R é a carga de ki.
- **Carga de ki (`mugen_charge.py`)**: segure **R** (Start). Mantém anim de início, loop e fim (`startAnim`, `anim`, `endAnim`), `startSound` e `sound` nos quadros originais (`soundAt`). Estado 3920 (loop) e 3921 (fim).
- **Golpes de voo e rotas (`mugen_routes.py`)**: grupos com `ctrl` ganham; grupos que só disparam do estado de voo viram `when: "fly"` (começa e volta para 3900 sem cair); grupos só de cancel são ignorados. Normais de chão (200-240) não são mais trocados por golpes de voo.
- **Cadeias (`mugen_chain.py`)**: até 12 estados por cadeia; `time = N` repete/corta o loop até N ticks; saída por pouso vira `landAt` (anima o pouso no lugar, sem cair no estado 52). `HitDef p1stateno` vira `followUp on: "hitNow"`. Sem limite de 6 hits; `Clsn1Default` lido; `fall.recover = 0` vira `recover: false`.
- **Comandos repetidos**: estado cujo comando já é tomado por outro grupo anterior fica como `shadowed` (também nunca dispara no MUGEN). Follow-ups saem com `input: ""` e não aparecem na lista de golpes.
- **`coverage.md`**: o conversor grava em `<saida>/coverage.md` o destino de cada `Statedef` (golpe, habilidade, script, comum do motor, shadowed, código morto, parcial ou NOT MIGRATED), contagem de sprites/sons e controladores descartados. Zero `NOT MIGRATED` é a meta; leia os parciais.
- **Paridade com evidência**: `UME_CHAR_DIR=<pasta-mugen> UME_TEST_UMF_JSON=<saida>/<id>.json UME_EVIDENCE_DIR=<dir> cargo test --release -p ume-engine --test umf_parity_evidence -- --nocapture` roda original vs UMF (intro, carga, Genki Dama, teleporte, meteoro, kamehameha) e grava `parity.md` + PNG lado a lado. Os sprites do UMF são lidos de `assets/chars/sprites/<id>` (link simbólico local, nunca commitar).
- **Kits reutilizáveis**: para reduzir JSON em lutadores novos ou editados, use `"extends": ["core", "ki_fighter", "clone_arts", "aerial_rush"]` (`crates/native/src/kits/`).
- **Schema**: o formato está em `docs/schema/umf-2.schema.json` (chaves desconhecidas são rejeitadas). Confira o pacote gerado com `node scripts/umf-schema.mjs <saida>/<id>.json`; o pacote Goku SSJ4 do `mugen-assets` passa. Mudou uma struct serde em `crates/native/src`? Atualize o schema no mesmo PR (`node --test scripts/umf-schema.test.mjs`).
- **`--extra` e `--report`**: `--extra` mescla JSON por cima (moves somados; `abilities`, `profile`, `helpers`, `passives` e `scripts` mesclados por chave, sem apagar a carga/voo detectados); `--report` imprime o resumo de cobertura de features convertidas.

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

Lua/ZSS arbitrário, helpers com lógica própria, `Statedef -2/-3` customizados e estados de IA: recrie como `passives`, `forms` ou deixe o personagem em formato legado. Nunca copie nomes de arquivo de terceiros ou arte; crie poses novas ou reuse as `anims` existentes do nativo.

## Regras do repositório

Sem comentários em código; arquivos de código até 250 linhas; PR com título `type: description`; teste de fumaça no navegador (wasm) antes de entregar.

## Biblioteca de habilidades

Antes de copiar controladores para um lutador, veja `docs/wiki/pt-BR/Biblioteca-de-Habilidades.md`: blocos (`blink`, `vfx`, `quake`, `projectile.repeat`) e o kit `abilities` (`"extends": ["abilities"]` + `"use": "ki_barrage"`) deixam o JSON do lutador curto. Padrão repetido em duas conversões vira bloco novo no motor, com teste.

## Aprender antes de migrar

Para entender como um pacote foi feito (comandos, hitdefs, helpers, projéteis, formas), rode a skill `mugen-dissector` primeiro; a tabela de padrões e os números de referência estão em `docs/wiki/pt-BR/Dissecacao-MUGEN.md`.
