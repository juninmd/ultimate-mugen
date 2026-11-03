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

## State controllers suportados

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `CtrlSet` `DefenceMulSet` `DestroySelf` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `GameMakeAnim` `Gravity` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `ModifyExplod` `MoveHitReset` `NotHitBy` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RemoveExplod` `ScreenBound` `SelfState` `SprPriority` `StateTypeSet` `SuperPause` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `Width`

`StopSnd` é aceito mas ainda não faz nada. Faltam (planejado): `ReversalDef` `HitOverride` `RemapPal` `SndPan` `VictoryQuote` `Null` `Offset` `FallEnvShake` `ForceFeedback` e os de clipboard.

## Triggers suportados

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Faltam (planejado): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

Funções extras: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

## Redirecionamentos

`parent`, `root`, `helper(id)`, `target(id)`, `partner`, `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Exemplo: `trigger1 = enemynear, life < 200`.

## Expressões

- Inteiro vs. float como no M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Intervalos: `stateno = [200, 299]`, `(200, 299]`.
- Operadores: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` e `:=` em `var(n) := valor`.

## Depurar

- `F2` no treino mostra hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <pasta>` lista estados e expressões com erro.
