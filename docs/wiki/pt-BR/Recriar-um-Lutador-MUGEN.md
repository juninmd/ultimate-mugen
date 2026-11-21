# Recriar um lutador M.U.G.E.N como UMF

Receita usada na reconstrução do Captain Marlow Reyes, baseada na [dissecação de quatro lutadores](Dissecacao-MUGEN.md). Transforma o que um pacote `.cmd`/`.cns`/`.air` faz em dados de um único `assets/chars/<id>.json`. A arte continua só PNG: a reconstrução reaproveita os frames de pixel art do lutador e nunca desenha um frame por código.

## Passos

1. Dissecar o pacote: `python3 -I .agents/skills/mugen-dissector/scripts/dissect.py summary <pasta>` para arquivos, comandos, estados e controllers, e `... hitdefs <arquivo.cns>` para a tabela de normais.
2. Listar as features, não o código: normais e cadeias, um projétil por botão, anti-aéreo, agarrão, contra-ataque, teleporte, helper ou clone, forma, supers.
3. Escolher os frames: só poses com PNG em `assets/chars/sprites/<id>/`. Um frame dado como pose de juntas inline é arte procedural e não é permitido nas anims.
4. Escrever as anims como `{pose, ticks, hit?}` com os ticks da tabela da dissecação e depois os golpes. `at` é o frame (base 1) que carrega o acerto, projétil ou velocidade.
5. Manter o primeiro especial como quarto de círculo mais `x` e entradas de carga só em golpes de chão: o smoke test do elenco e a auditoria de entradas dependem disso.
6. Rodar as auditorias de [Dev tools](Dev-Tools.md), capturar o showcase de golpes e o smoke test no navegador.

## Mapeamento M.U.G.E.N para UMF

| Padrão M.U.G.E.N | Dado UMF |
|---|---|
| `Statedef 200..610` com `HitDef` | `moves[]` com `kind: normal`, `when`, `hits[].pausetime`, `hits[].hitstun` |
| cancels por `movehit` e `numhelper` | `cancels`, `rekka {after, window, contact}` |
| `Projectile` com `projhits` e `projpriority` | `projectile {at, speed, hits, priority, removeTime}` |
| Statedef de bola no ar | Automático: todo especial de chão ganha versão aérea |
| Carga `~60$B,F,a` | `"input": "~40$B, F, x"` em golpe só de chão |
| `PosAdd p2dist x + 40` com `Turn` (Shunpo) | `teleport: <frame>`, `cost`, `invuln` |
| Contra-ataque `ReversalDef` / `HitOverride` | `parry {window}` |
| Agarrão `TargetBind` | `throw {damage, bind, launch}` ou `throw.steps` |
| Clone helper (Aizen) | `helpers.clones` |
| Modo de poder `var(11)` (Kaio Ken) | `abilities.forms` |
| `SuperPause`, `AfterImage`, `EnvShake` | `kind: super`, `fx`, `shake` |
| Constantes `[Data]`/`[Velocity]` | `stats` |

## Captain Marlow Reyes reconstruído

Tudo em `assets/chars/marlow.json`, só dados:

| Mudança | Origem da ideia |
|---|---|
| Todas as poses de juntas inline removidas das anims; 13 frames agora usam poses PNG | Regra só sprites |
| Normais leves com pausa 6/7 e 12 ticks de stun, fortes com pausa 11/12 e 15 | Tabela de normais dos quatro pacotes |
| `tide_step` (`~D, DF, F, c`, 200 de power): teleporta atrás do oponente no frame 3, cancela em `riptide` ou `spout` | Shunpo do Aizen; no botão livre `c` (o Aizen usa Z para o Shunpo) porque `D, D, a` perde para os golpes agachados e uma meia-lua contém os quartos de círculo dos outros especiais |
| `flurry` (`~D, DF, F, a`): cinco golpes leves e finalização que lança, avançando | Rush 1050 do Goku SSJ |
| `broadside` (`~B, F, x`): projétil de três acertos e prioridade 2, em movimento rápido trás-frente e não em carga, porque todo especial ganha versão aérea e a auditoria de entradas reais não alcança a versão aérea de uma carga de 40 ticks | Bolas carregadas do Goku SSJ |
| `helpers.clones` `Phantom Crew` (`F, B, F`, 500 de power): cópia autônoma que persegue e ataca; entrada só de direções porque o bloco de invocação é compilado depois de todos os golpes, então qualquer entrada com botão é tomada antes por um normal, e ela só começa em pé | Clone do Aizen (estado 700) |
| `stats.airDashes = 1` | Mobilidade aérea de Aizen e Gogeta |

Status da arte: nenhuma arte nova foi criada. O conjunto de frames do Marlow não tem poses dedicadas para o rush e o teleporte (poses reaproveitadas e retemporizadas); um conjunto de frames próprio para `flurry` e `tide_step` é a pendência de arte de quem recria os sprites.
