# Scripts CNS

Toda lógica de luta, inclusive a dos lutadores UMF, roda como CNS: estados (`[Statedef]`), controllers (`[State]`) e triggers. Para programar um lutador à mão, use o formato M.U.G.E.N (pasta com `.def`, `.cns`, `.cmd`) e coloque em `mugen/chars/`.

Sintaxe moderna ZSS (do Ikemen-GO): **planejado**, compilando para o mesmo bytecode.

## Arquivo `.def`

```ini
[Info]
name = "Meu Lutador"
displayname = "Meu Lutador"
author = "você"
localcoord = 320, 240

[Files]
cmd = meu.cmd
cns = meu.cns
st = meu.st
stcommon = common1.cns
sprite = meu.sff
anim = meu.air
sound = meu.snd
pal1 = meu1.act
```

`localcoord = 1280, 720` marca personagem HD; a engine escala sozinha.

## Exemplo de estado

```ini
[Statedef 1000]
type = S
movetype = A
physics = S
anim = 1000
ctrl = 0
velset = 0, 0
poweradd = 30

[State 1000, golpe]
type = HitDef
trigger1 = AnimElem = 3
attr = S, SA
damage = 90, 10
animtype = Heavy
guardflag = MA
pausetime = 12, 12
sparkno = 2
ground.type = High
ground.hittime = 18
ground.velocity = -5
air.velocity = -4, -6
fall = 1

[State 1000, fim]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
```

E no `.cmd`:

```ini
[Command]
name = "hadou"
command = ~D, DF, F, x
time = 20

[Statedef -1]

[State -1, hadou]
type = ChangeState
value = 1000
triggerall = command = "hadou"
trigger1 = statetype = S && ctrl
```

Extras de comando (Ikemen GO 1.0):

| Recurso | Comportamento |
|---|---|
| Passo `N` | Direção neutra: nenhuma direção pressionada (`command = N, x`, `~D, N, x`, `N+x`). O passo neutro é um estado mantido, como `/`, e vale enquanto nenhuma direção está pressionada |
| `pausebuffer` | `[Command]` (ou `command.pausebuffer` em `[Defaults]`) mantém ativo por esse número de ticks extras um comando disparado durante hit pause, hit shake ou pause/super pause, para que possa ser lido quando a pausa terminar. Também aceita `buffer.pausetime`. Padrão 0 |
| Estados customizados | Quando um jogador roda um estado de outro personagem (`TargetState`, `p2getp1state`), `Command` usa os comandos do dono do estado contra a entrada do jogador no estado |

## State controllers suportados

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AppendToClipboard` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `ClearClipboard` `CtrlSet` `DefenceMulSet` `DestroySelf` `DisplayToClipboard` `DizzyPointsAdd` `DizzyPointsSet` `DizzySet` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `FallEnvShake` `ForceFeedback` `GameMakeAnim` `Gravity` `GuardBreakSet` `GuardPointsAdd` `GuardPointsSet` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitOverride` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `MapAdd` `MapReset` `MapSet` `ModifyExplod` `ModifyProjectile` `ModifySnd` `MoveHitReset` `NotHitBy` `Null` `Offset` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RedLifeAdd` `RedLifeSet` `RemapPal` `RemoveExplod` `RemoveText` `ReversalDef` `RootVarAdd` `RootVarSet` `ScreenBound` `SelfState` `SndPan` `SprPriority` `StateTypeSet` `StopSnd` `SuperPause` `TagIn` `TagOut` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Text` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `VictoryQuote` `Width`

Também suportados: `Zoom` `Camera` `GroundLevelOffset` `Depth` `DepthBound` `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.
Limites dos controladores mais simples: `Offset` desloca somente o sprite desenhado, nunca as caixas de colisão nem a posição; `Null` não faz nada; `VictoryQuote` guarda o seu `value` no lutador para a apresentação da frase de vitória; `ForceFeedback` vira um evento de vibração para o personagem (`self = 1`) ou para o oponente; `StopSnd` e `SndPan` agem nos canais de som iniciados pelo personagem; `DisplayToClipboard`, `AppendToClipboard` e `ClearClipboard` guardam as últimas 10 linhas no lutador (`%d`, `%i`, `%f`, `%%`, `\n` e `\t` no texto, até 5 `params`).

Notas de tremor e som: `EnvShake` (e os parâmetros `envshake.*` / `fall.envshake.*` do HitDef) aceitam `dir`: `0` (padrão) balança o cenário nos dois eixos, `1` só na horizontal e `2` só na vertical; valores acima de 2 contam como 2. `PlaySnd` aceita `stopongethit` e `stoponchangestate`. Um som marcado é interrompido quando o personagem que o iniciou apanha (ou defende) ou entra em outro state; um novo `PlaySnd` no mesmo `channel` substitui as marcas anteriores, e `StopSnd` continua parando o canal. As marcas viajam junto com o evento de som: a engine só as informa e quem interrompe é a interface.

Notas sobre Explod: `ignorehitpause` vale 1 por padrão (use 0 para congelar junto com o hit pause do dono). `postype = left/right/front/back` e `space = screen` mantêm o explod fixo na tela enquanto a câmera se move. `ModifyExplod` aceita `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` e `trans`; `facing` é relativo ao dono. `ExplodBindTime` só altera o tempo de vínculo.

Efeitos de paleta empilham nesta ordem: `BGPalFX` do cenário, depois `AllPalFX`, depois o `PalFX` do próprio lutador. Eles valem no tick em que mudam (sem quadro de atraso), e um trans `Sub` ignora o tom da paleta enquanto o trans aditivo o mantém. `hue` (graus) gira as cores em torno do eixo cinza e `sinhue` oscila essa rotação usando o período do `sinadd`; os dois funcionam nos três controllers de paleta.

Canais de som: `PlaySnd` aceita `loop`, `loopstart`/`loopend` (milissegundos), `loopcount` (0 = infinito), `stopongethit`, `stoponchangestate`, `pan`/`abspan`, `volumescale` e `freqmul`; `ModifySnd` reajusta `channel`, `pan`, `volumescale` e `loopstart`/`loopend` de um canal vivo. Formato de música não suportado gera aviso e cai para a trilha embutida (trackers são rejeitados por decisão, veja [decisões de compatibilidade](Decisoes-de-Compatibilidade.md)).

Parâmetros de vida do HitDef: `redlife` aumenta a faixa de red life do defensor, `dizzypoints` e `guardpoints` drenam os medidores de tontura e guarda (zero ativa `Dizzy`/`GuardBreak` e entra no estado 5300/5400 quando o personagem os define), `score` alimenta o placar do atacante e do defensor, `sparkscale` escala o spark do golpe, `missonoverride` mostra o texto `MISS` quando um `HitOverride` absorve o golpe, `unhittabletime` dá N ticks de invencibilidade ao defensor (a contagem pausa durante o hit pause) e `xaccel` desloca o defensor na horizontal enquanto estiver no estado de dano. `attack.depth` é lido e ignorado (a engine é 2D). `p2clsncheck = 0` deixa o golpe conectar sem caixas do defensor e `p2clsnrequire = n` exige a caixa Clsn2 `n` do defensor (índice -1 = qualquer).

Notas de helper: um helper congela durante `Pause` e `SuperPause`, a menos que ainda tenha ticks de `pausemovetime` / `supermovetime` ou tenha nascido com `pausebool = 1`, que o faz ignorar qualquer pausa. Helpers com `keyctrl = 1` só recebem os comandos do root enquanto o round é lutado; decidido o round, não recebem nenhum. Na tela de vitória (estado de round 4) os helpers ficam congelados e ocultos.

## Controllers Ikemen, de cenário e de jogador

`PlayBGM` `StopBGM` `ModifyBGM` `ModifyStageVar` `ModifyStageBG` `ModifyBGCtrl` `ModifyPlayer` `PrintToConsole` `RunState` `Height` `GetHitVarSet` `ModifyHitDef` `ModifyReversalDef` `OverrideClsn` `ResizeClsn` `TransformClsn` `TargetAdd` `TargetScoreAdd` `TargetRedLifeAdd` `TargetDizzyPointsAdd` `TargetGuardPointsAdd` `ScoreAdd` `RoundTimeSet` `RoundTimeAdd` `AssertCommand` `AssertInput` `ShiftInput` `RemapSprite` `TransformSprite` `MatchRestart`

Faltam (planejado ou rejeitado por decisão, interpretados e ignorados com um aviso): `Storyboard` `RemoveStoryboard` `Dialogue` `SaveState` `LoadState` `LifebarAction` `ChangeMovelist` `LoadFile` `SaveFile` `ShaderSet`

Parâmetros e comportamento em [Ikemen extensions](../Ikemen-Extensions.md).

## Triggers suportados

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimElemVar` `AnimExist` `AnimLength` `AnimPlayerNo` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Clamp` `ClsnOverlap` `ClsnVar` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DisplayName` `Dizzy` `DizzyPoints` `DizzyPointsMax` `DrawGame` `E` `Exp` `ExplodVar` `facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `GuardBreak` `GuardPoints` `GuardPointsMax` `HelperIndexExist` `HelperVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `ID` `IfElse` `IkemenVersion` `Index` `InGuardDist` `InputTime` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `LoseKO` `LoseTime` `MajorVersion` `Map` `MatchNo` `MatchOver` `Max` `Min` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `MugenVersion` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `NumText` `P1Dist` `P1Name` `P2BodyDist` `P2Command` `P2Ctrl` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2Power` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalFXVar` `PalNo` `ParentDist` `Physics` `Pi` `PlayerIDExist` `pos` `Power` `PowerMax` `PrevStateNo` `ProjCancelTime` `ProjClsnOverlap` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `ProjVar` `Random` `RandomRange` `RedLife` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `RunOrder` `ScreenPos` `SelfAnimExist` `SelfCommand` `Sign` `Sin` `SpritePlayerNo` `SpriteVar` `Sqrt` `Standby` `StateNo` `StateTime` `StateType` `SysFVar` `SysVar` `Tan` `TeamMap` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `vel` `Win` `WinKO` `WinPerfect` `WinTime` `AirJumpCount` `Alpha` `Angle` `BotBoundBodyDist` `BotBoundDist` `Const1080p` `HitByAttr` `HitDefVar` `HitOverridden` `HitVel` `InCustomAnim` `InCustomState` `InRunState` `IntroState` `IsAsserted` `JugglePoints` `LastPlayerID` `LayerNo` `LocalCoord` `MemberNo` `MoveCountered` `MoveHitVar` `NumPlayer` `NumStageBG` `OutroState` `ParentExist` `PlayerIndexExist` `PlayerNo` `PrevAnim` `PrevMoveType` `PrevStateType` `Scale` `ScreenHeight` `ScreenWidth` `SelfStateNoExist` `SprPriority` `StageBackEdgeDist` `StageBGVar` `StageConst` `StageFrontEdgeDist` `StageVar` `TeamLeader` `TeamSize` `TopBoundBodyDist` `TopBoundDist` `XAngle` `XShear` `YAngle` `ZoomVar` `Attack` `Defence` `AttackMul` `DefenceMul` `ReceivedDamage` `ReceivedHits` `ComboCount` `Score` `ScoreTotal` `Round` `RoundTime` `TimeRemaining` `TimeElapsed` `TimeTotal` `FightTime` `RoundsWon` `FirstAttack` `DecisiveRound`

Notas de projétil: o projétil guarda o lado do time e o jogador raiz no momento em que nasce. Virar, ou o dono mudar de lado depois, não altera quem ele pode acertar. `NumProj` e `NumProjID` contam todos os projéteis do root do jogador, inclusive os disparados por helpers, e continuam contando depois que o helper é destruído. Um projétil cujo helper sumiu ainda acerta e é creditado ao root.

Faltam (planejado): `PlayerID`.

## Triggers de borda da tela

`LeftEdge` e `RightEdge` são absolutos: sempre devolvem a borda esquerda e a direita da tela. `FrontEdge` e `BackEdge` dependem de `Facing`: quem olha para a direita lê `FrontEdge = RightEdge` e `BackEdge = LeftEdge`, e quem olha para a esquerda lê o contrário. Os quatro usam as unidades do `localcoord` do personagem e as mesmas bordas de `FrontEdgeDist` e `BackEdgeDist`, então `(FrontEdge - X) * Facing` é igual a `FrontEdgeDist` (com `X` a posição do personagem no mundo).

## Distâncias entre localcoords diferentes

Todo trigger de posição e distância converte o resultado do espaço do mundo para o localcoord do personagem que o avalia (`P2Dist`, `P2BodyDist`, `RootDist`, `ParentDist`, `ScreenPos`, `LeftEdge`/`RightEdge`/`FrontEdge`/`BackEdge` e a família `...EdgeDist`). Numa luta 1280x720 contra 320x240, o mesmo vão aparece 4 vezes maior para o personagem HD. `P2BodyDist` subtrai as larguras frontais dos dois depois de convertê-las para o mundo, então vale 0 quando as caixas se tocam, seja qual for o localcoord de cada personagem. `P2Dist` e `P2BodyDist` dependem apenas das duas posições: a câmera nunca interfere, inclusive nos estados customizados de um arremesso.
## Zoom da câmera

`Zoom` muda a escala da câmera (1 = normal, acima de 1 = mais perto). A câmera é determinística: avança um passo por tick, sem relógio.

| Parâmetro | Padrão | Significado |
| --- | --- | --- |
| `scale` | 1 | Escala pedida |
| `time` | 1 | Ticks em que o pedido continua ativo; dispare a cada tick para mantê-lo |
| `delta` | 0 | Somado à escala pedida a cada tick do pedido (uma rampa) |
| `pos` | 0, 0 | Deslocamento do foco da câmera no localcoord do jogador |
| `lag` | 0 | 0 vai à escala pedida em um tick; valores próximos de 1 suavizam (`1 - lag` da distância restante por tick) |
| `zoomindelay` | 0 | Ticks de espera antes de um zoom-in (`scale > 1`) começar |
| `bound` | 1 | 1 mantém a escala dentro dos limites do cenário (`zoomin` como maior escala, `1 / zoomout` como menor, ampliada pelo zoom-out de equipes) e o deslocamento dentro dos limites do cenário; 0 só limita a escala a 0.25..4 e ignora os limites do cenário |

Quando o pedido termina, a câmera volta à escala normal usando `zoominspeed` / `zoomoutspeed` do cenário.

`CameraZoom` devolve a escala atual da câmera. `ZoomVar(<nome>)` expõe o estado do zoom: `scale` (atual), `target` (escala pedida), `active`, `time` (ticks restantes), `posx`, `posy`, `lag`, `delta`, `delay`, `bound`, `min` e `max` (limites de escala permitidos pelo cenário agora). Todos os nomes valem 0 (ou a escala atual em `target`) quando nenhum `Zoom` está ativo.

## Controladores de câmera e nível do chão

O motor é 2D, então não há eixo Z: `Pos Z`, `Vel Z` e os outros componentes Z sempre valem 0.

- `Camera` (`pos`, `zoom`, `bound`, `lag`, `time`): move o foco da câmera por `pos` (localcoord do jogador) e, opcionalmente, define a escala da câmera com `zoom`. Usa o mesmo mecanismo do `Zoom` (veja acima): `bound = 1` mantém o deslocamento dentro dos limites do cenário e a escala dentro dos limites de zoom do cenário, `lag` suaviza o zoom, `time` é o número de ticks em que o pedido fica ativo. Sem `zoom`, a escala atual é mantida.
- `GroundLevelOffset value = n`: eleva em `n` pixels o chão sob este personagem (negativo cava), que passa a ficar sobre o chão elevado com pouso, física de chão e detecção de golpe no chão/ar normais. `Pos Y` é medido a partir do chão do próprio personagem (0 enquanto está sobre ele); `ScreenPos Y` e a altura de criação de projéteis, explods e helpers incluem o deslocamento. O deslocamento zera no início do round.
- `Depth` e `DepthBound` são aceitos mas ignorados (Z está fora do escopo); o primeiro uso imprime um aviso (`depth is accepted but ignored: the engine has no Z axis`) e o personagem não é alterado.

## Triggers de tela, câmera e cenário

Todas as distâncias e tamanhos usam o localcoord do jogador que avalia. Triggers "relativos ao zoom" mudam quando a câmera dá zoom (o zoom-out de 5v5, `Zoom`, `Camera`).

| Trigger | Relativo ao zoom | Significado |
| --- | --- | --- |
| `ScreenWidth`, `ScreenHeight` | sim | Tamanho da área visível (`GameWidth` / `GameHeight` continuam devolvendo o tamanho bruto da vista em pixels do jogo) |
| `TopBoundDist` | sim | Distância da posição do jogador até o ponto mais alto que o cenário deixa a câmera mostrar (`boundhigh`, elevado por `dynamicboundhigh`) |
| `TopBoundBodyDist` | sim | O mesmo, menos o `size.height` do jogador (cabeça até o teto) |
| `BotBoundDist`, `BotBoundBodyDist` | sim | Distância dos pés até o ponto mais baixo que a câmera mostra (`boundlow` mais a altura da vista); iguais porque a origem fica nos pés |
| `StageFrontEdgeDist`, `StageBackEdgeDist` | sim | Distância até os limites laterais do cenário (`boundleft` / `boundright` ampliados por meia vista) à frente / atrás do jogador, conforme `Facing` |
| `NumStageBG` / `NumStageBG(id)` | não | Número de camadas de fundo do cenário, opcionalmente só as com o `id` dado |
| `StageBGVar(id, nome)` | não | Valor atual da primeira camada com esse `id`: `pos.x`, `pos.y` (início mais rolagem e senoide), `start.x/y`, `delta.x/y`, `velocity.x/y`, `tile.x/y`, `layerno`, `id`, `visible`, `enabled`, `sprite.group`, `sprite.index`; id ou nome desconhecido dá 0 |
| `StageConst(nome)` | não | Número do grupo `[Constants]` do cenário (0 se faltar ou nos cenários nativos) |
| `Const1080p(n)` | não | `n` convertido de um desenho com 1920 pixels de largura para o localcoord do jogador |
| `LocalCoord X` / `LocalCoord Y` | não | Largura / altura do localcoord do jogador |
| `Angle`, `Scale X/Y`, `Alpha X/Y`, `SprPriority`, `LayerNo` | não | O estado de desenho atual do jogador (`Alpha` é origem / destino, 256 / 0 quando opaco). `XAngle`, `YAngle` e `XShear` valem 0: o renderizador 2D não tem inclinação nem cisalhamento |
| `ZoomVar(nome)`, `CameraZoom` | sim | Veja "Zoom da câmera" |

## Redirecionamentos

`parent` `root` `helper` `target` `partner` `enemy` `enemynear` `playerid` `p2` `stateowner`

`helper(id)` e `target(id)` aceitam um ID opcional; `partner(n)` é o n-ésimo outro jogador não-helper do mesmo lado (padrão 0; um helper redireciona pelo seu root; NaN quando não existe); `enemy(n)` e `enemynear(n)` escolhem o n-ésimo oponente. Exemplo: `trigger1 = enemynear, life < 200`.

## Expressões

- Inteiro vs. float como no M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Intervalos: `stateno = [200, 299]`, `(200, 299]`.
- Operadores: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` e `:=` em `var(n) := valor`.

## Regras de pausa e congelamento

| Entidade | Hit pause (atacante) / hit shake (defensor) | `Pause` | `SuperPause` |
|---|---|---|---|
| Lutador ou helper | Animação, `Time`, física e timers param; só rodam controllers com `ignorehitpause = 1` | Congelado, exceto quem pausou durante `movetime` e helpers com ticks de `pausemovetime` restantes | Congelado, exceto quem pausou durante `movetime` e helpers com ticks de `supermovetime` restantes |
| Explod | Continua com `ignorehitpause = 1` (padrão); com `0` para enquanto o dono está em hit pause ou hit shake, inclusive o vínculo de posição | Congelado, salvo ticks de `pausemovetime` restantes | Congelado, salvo ticks de `supermovetime` restantes |
| Projétil | Não é afetado pelo hit pause do dono | Congelado, salvo ticks de `pausemovetime` restantes | Congelado, salvo ticks de `supermovetime` restantes |

- O orçamento de `pausemovetime`/`supermovetime` gasta um tick por frame congelado e só pelo seu próprio tipo de pausa.
- `AnimTime`, `AnimElemTime`, `AnimElemNo` e `Time` mantêm o valor durante todo o hit pause ou shake e voltam no primeiro tick livre.
- `HitPauseTime` é o hit pause restante do atacante; `GetHitVar(hitshaketime)` e `HitShakeOver` descrevem o shake do defensor. Os demais campos de `GetHitVar` são uma captura feita no momento do golpe e não mudam durante o shake; o relógio de recuperação (`HitOver`) também espera o fim do shake.
- Testes de regressão: `crates/engine/tests/freeze_rules.rs`.

## Depurar

- `F2` no treino mostra hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <pasta>` lista estados e expressões com erro.

## Diagnóstico no carregamento

Cada carregamento de personagem confere os controladores, triggers e parâmetros lidos com o que a engine realmente implementa. Não há lista fixa: cada nome desconhecido é executado uma vez numa partida descartável e marcado quando o despacho não o trata, então o relatório acompanha a engine conforme controladores e triggers são adicionados.

O relatório indica, por arquivo e estado (`.cns`, `.cmd`, `.st`, `.zss`):

| Achado | Significado |
|---|---|
| controlador não suportado | o `type` não tem implementação |
| trigger desconhecido | o nome do trigger não é avaliado pela engine |
| redirecionamento / trigger de texto não suportado | um tipo de redirecionamento (`partner`, ...) ou comparação de texto (`p3name = ...`) que a engine não resolve |
| parâmetro desconhecido | nenhum caminho do controlador lê esse parâmetro |
| flag de AssertSpecial desconhecida | a flag não é reconhecida |

Cada carga imprime um resumo no stderr (`n controladores não suportados, m triggers desconhecidos, ...`), sem repetição por nome; `TRACE=1` acrescenta os nomes com os arquivos e estados que os usam. Defina `UME_QUIET_DIAGNOSTICS=1` para silenciar.

O `mugen_check` imprime o resumo de cada personagem e, com `--strict`, imprime os detalhes e termina com código diferente de zero se algum personagem tiver achados:

```bash
cargo run --release -p ume-engine --example mugen_check -- --strict mugen/chars/kfm
```

O CI roda com `--strict` sobre `crates/engine/tests/data/chars`; um recurso usado pelos personagens de teste que a engine não implementa quebra o build. No código, `ume_engine::diagnostics::audit_cns(arquivo, texto)` devolve os achados de um trecho e `CharDef::diagnostics()` os de um personagem carregado.

## Triggers de introspecção

Triggers que leem de volta o que o personagem criou ou o que ele está desenhando. O último argumento é o nome da variável escrito sem aspas, como no Ikemen GO; um eixo é uma segunda palavra (`pos x`). Uma consulta que não encontra nada (explod, projétil ou caixa inexistente) é indefinida (NaN): qualquer comparação com ela continua indefinida e conta como falsa, então teste a existência com `NumExplod`, `NumProj` ou `HelperIndexExist` quando a diferença importar. Posições e velocidades usam as coordenadas do próprio personagem; as posições seguem a convenção de `Pos` (`x` relativo à câmera, `y` relativo ao chão).

| Trigger | Variáveis |
|---|---|
| `ExplodVar(id, índice, var)` | `id` `time` `removetime` `bindtime` `bindid` `anim` `animelem` `animelemtime` `animtime` `animplayerno` `spriteplayerno` `facing` `ignorehitpause` `layerno` `pausemovetime` `supermovetime` `sprpriority` `pos x/y/z` `vel x/y/z` `accel x/y/z` `scale x/y`. Lê o `índice`-ésimo explod do chamador com esse `id` (`id` abaixo de 0 vale qualquer um) |
| `ProjVar(id, índice, var)` | `projid` `time` `projremovetime` `projhits` `projmisstime` `projpriority` `projsprpriority` `projedgebound` `projstagebound` `lowbound` `highbound` `projhitanim` `projremanim` `projcancelanim` `projremove` `anim` `animelem` `facing` `teamside` `pausemovetime` `supermovetime` `pos x/y/z` `vel x/y/z` `accel x/y/z` `velmul x/y` `remvelocity x/y/z` `scale x/y`. Lê o `índice`-ésimo projétil vivo do chamador com esse `id` |
| `PalFXVar(var)` | `time` (ticks restantes) `add.r/g/b` `mul.r/g/b` `color` `invertall`; prefixo `bg.` ou `all.` para os efeitos de fundo e global. Sem efeito ativo vale a identidade (`add` 0, `mul` 256, `color` 256) |
| `ClsnVar(tipo, índice, borda)` | `tipo` é `clsn1`, `clsn2`, `size` ou `dummy`; `borda` é `back`, `front`, `top` ou `bottom`, relativa ao personagem e à sua direção. `size` é a caixa de corpo de `size.ground.back/front` e `size.height`; `dummy` não tem caixas |
| `ClsnOverlap(tipo, playerid, tipo)` | agora aceita `size` (nos dois lados) e `dummy` (no primeiro, nunca sobrepõe) além de `clsn1` e `clsn2` |
| `ProjClsnOverlap(índice, playerid, tipo)` | a caixa de ataque do `índice`-ésimo projétil do chamador contra uma caixa daquele jogador |
| `HelperVar(var)` | num helper: `id` `helpertype` `keyctrl` `ownpal` `preserve`; indefinido fora de um helper |
| `SpriteVar(var)` | o sprite atual: `group` `image` `width` `height` `xoffset` `yoffset` |
| `AnimElemVar(var)` | o elemento atual da animação: `group` `image` `time` `hflip` `vflip` `xoffset` `yoffset` `xscale` `yscale` `angle` `alphasource` `alphadest` `numclsn1` `numclsn2` |
| `InputTime(tecla)` | ticks em que a tecla está pressionada: `U` `D` `L` `R`, relativas `F` `B`, `N` (nada pressionado) e os botões `a` `b` `c` `x` `y` `z` `s` (maiúscula importa: `B` é trás, `b` é o botão) |
| `AnimLength` `AnimPlayerNo` `SpritePlayerNo` | total de ticks da animação (-1 se nunca termina); número do jogador dono da animação e dos sprites |
| `Index` `RunOrder` `HelperIndexExist(n)` | posição do personagem entre o root e seus helpers (o root é 0); ordem de atualização na partida, começando em 1; se o root tem o `n`-ésimo helper (base 0) |
| `IkemenVersion(major)` `IkemenVersion(minor)` | versão do comportamento do Ikemen GO seguida: 1 e 0 |
| `DisplayName = "x"` `SelfCommand = "x"` | comparações de texto com o nome de exibição e com o buffer de comandos do próprio personagem |

Ainda não implementados, e apontados pelo diagnóstico de carregamento: `BGMVar`, `SoundVar` (o núcleo da engine não guarda estado de áudio), `AnimLoopCount`, `AnimPrefix`, `HelperName`, as variáveis `angle`, `friction`, `xshear`, `drawpal.*` e `shader` de `ExplodVar`, e `attr`, `guardflag`, `hitflag`, `shadow` de `ProjVar`. Um nome de variável desconhecido é apontado uma vez por carregamento como `explodvar(nome)` e vale 0.
