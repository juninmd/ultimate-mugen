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

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `CtrlSet` `DefenceMulSet` `DestroySelf` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `GameMakeAnim` `Gravity` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `ModifyExplod` `MoveHitReset` `NotHitBy` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RemoveExplod` `ScreenBound` `SelfState` `SprPriority` `StateTypeSet` `SuperPause` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `Width`

Também suportados: `Zoom` `Camera` `GroundLevelOffset` `Depth` `DepthBound` `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.

Notas sobre Explod: `ignorehitpause` vale 1 por padrão (use 0 para congelar junto com o hit pause do dono). `postype = left/right/front/back` e `space = screen` mantêm o explod fixo na tela enquanto a câmera se move. `ModifyExplod` aceita `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` e `trans`; `facing` é relativo ao dono. `ExplodBindTime` só altera o tempo de vínculo.

## Triggers suportados

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Command` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Faltam (planejado): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

`P3Name` a `P8Name` comparam com o jogador daquele slot (ordem de slot da partida, não de lado); um slot sem jogador nunca coincide. `P2Command` lê o buffer de comandos do oponente mais próximo, como `Command` faz para o próprio personagem.

Funções extras: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

`HitCount` soma os `numhits` de cada golpe que o atacante acerta. `UniqHitCount` conta cada par distinto `(alvo, id do HitDef)` uma única vez, então três golpes de um mesmo HitDef dão `HitCount = 3` e `UniqHitCount = 1`. Ambos são zerados juntos nas trocas de estado, a menos que `HitCountPersist = 1`.

`GetHitVar(<nome>)` lê os dados do último golpe sofrido (após `HitDef`, defesa ou `HitOverride`). Além dos nomes do MUGEN 1.1 (`type`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.*`, `kill`, `id`, `playerno`), estes nomes do Ikemen retornam valores reais: `recovertime` (ticks restantes até `fall.recovertime` permitir a recuperação, 0 fora de queda), `down.recover` (o `down.recover` do HitDef, padrão 1), `priority`, `facing` (direção do atacante, 1 ou -1), `power` (poder recebido pelo alvo), `isbound` (ao vivo, 1 enquanto o alvo está preso), `attr` (máscara de bits: S=1 C=2 A=4 L=8, NA=64 NT=128 NP=256 SA=512 ST=1024 SP=2048 HA=4096 HT=8192 HP=16384), `guardflag` (máscara de bits: H=1 L=2 A=4), `hitdamage`, `guarddamage`, `hitpower`, `guardpower`, `dizzypoints` (`dizzypoints` do HitDef, padrão o dano do golpe), `guardpoints` (`guardpoints` do HitDef, padrão o dano na defesa), `redlife` (`redlife` do HitDef, valor de golpe ou defesa), `score` (`score` do HitDef, valor de golpe ou defesa), `teamside` (lado do atacante, 1 ou 2), `projid` (-1 se não foi projétil), `playerid` (ID do atacante, igual a `id`), `p1stateno` e `p2stateno` (-1 quando o HitDef não define). Qualquer outro nome registra um aviso no stderr ao carregar a expressão e vale 0.

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

## Redirecionamentos

`parent`, `root`, `helper(id)`, `target(id)`, `partner(n)` (o n-ésimo outro jogador não-helper do mesmo lado, padrão 0; um helper redireciona pelo seu root; NaN quando não existe), `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Exemplo: `trigger1 = enemynear, life < 200`.

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
