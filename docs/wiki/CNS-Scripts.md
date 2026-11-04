# CNS scripts

All fight logic, including UMF fighters, runs as CNS: states (`[Statedef]`), controllers (`[State]`) and triggers. To program a fighter by hand, use the M.U.G.E.N format (a folder with `.def`, `.cns`, `.cmd`) and put it in `mugen/chars/`.

Modern ZSS syntax (from Ikemen GO) is **planned**, compiling to the same bytecode.

## `.def` file

```ini
[Info]
name = "My Fighter"
displayname = "My Fighter"
author = "you"
localcoord = 320, 240

[Files]
cmd = mine.cmd
cns = mine.cns
st = mine.st
stcommon = common1.cns
sprite = mine.sff
anim = mine.air
sound = mine.snd
pal1 = mine1.act
```

`localcoord = 1280, 720` marks an HD character; the engine scales it automatically.

## State example

```ini
[Statedef 1000]
type = S
movetype = A
physics = S
anim = 1000
ctrl = 0
velset = 0, 0
poweradd = 30

[State 1000, hit]
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

[State 1000, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
```

And in the `.cmd`:

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

## Supported state controllers

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `CtrlSet` `DefenceMulSet` `DestroySelf` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `GameMakeAnim` `Gravity` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `ModifyExplod` `MoveHitReset` `NotHitBy` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RemoveExplod` `ScreenBound` `SelfState` `SprPriority` `StateTypeSet` `SuperPause` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `Width`

Also supported: `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.

Explod notes: `ignorehitpause` defaults to 1 (set 0 to freeze with the owner's hit pause). `postype = left/right/front/back` and `space = screen` keep the explod fixed on screen while the camera moves. `ModifyExplod` accepts `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` and `trans`; `facing` is relative to the owner. `ExplodBindTime` only changes the bind time.

## Supported triggers

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Command` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Missing (planned): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

`P3Name` to `P8Name` compare against the player in that slot (slot order of the match, not side); an unused slot never matches. `P2Command` reads the command buffer of the nearest opponent, like `Command` does for the owner.

Extra functions: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

`HitCount` sums the `numhits` of every hit the attacker lands. `UniqHitCount` counts each distinct `(target, HitDef id)` pair once, so three hits of one HitDef give `HitCount = 3` and `UniqHitCount = 1`. Both reset together on state changes unless `HitCountPersist = 1`.

## Redirections

`parent`, `root`, `helper(id)`, `target(id)`, `partner(n)` (the n-th other non-helper player on the same side, default 0; a helper redirects through its root; NaN when there is none), `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Example: `trigger1 = enemynear, life < 200`.

## Expressions

- Integer vs float like M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Ranges: `stateno = [200, 299]`, `(200, 299]`.
- Operators: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` and `:=` in `var(n) := value`.

## Debugging

- `F2` in training shows hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <folder>` lists states and expressions with errors.
