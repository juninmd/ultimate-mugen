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

`GetHitVar(<name>)` reads the data of the last hit taken (after `HitDef`, guard or `HitOverride`). Beyond the MUGEN 1.1 names (`type`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.*`, `kill`, `id`, `playerno`) these Ikemen names return real values: `recovertime` (ticks left until `fall.recovertime` allows recovery, 0 when not falling), `down.recover` (the HitDef `down.recover`, default 1), `priority`, `facing` (attacker facing, 1 or -1), `power` (power the target received), `isbound` (live, 1 while the target is bound), `attr` (bit mask: S=1 C=2 A=4 L=8, NA=64 NT=128 NP=256 SA=512 ST=1024 SP=2048 HA=4096 HT=8192 HP=16384), `guardflag` (bit mask: H=1 L=2 A=4), `hitdamage`, `guarddamage`, `hitpower`, `guardpower`, `dizzypoints` (HitDef `dizzypoints`, default the hit damage), `guardpoints` (HitDef `guardpoints`, default the guard damage), `redlife` (HitDef `redlife`, hit or guard value), `score` (HitDef `score`, hit or guard value), `teamside` (attacker side, 1 or 2), `projid` (-1 unless a projectile hit), `playerid` (attacker ID, same as `id`), `p1stateno` and `p2stateno` (-1 when the HitDef sets none). Any other name logs one warning on stderr when the expression is loaded and evaluates to 0.

Combat and state triggers (localcoord-scaled where they return positions or velocities):

| Trigger | Result |
|---|---|
| `HitDefVar(<name>)` | Reads the character's last `HitDef`: `p1stateno`, `p2stateno`, `priority`, `sparkx`, `sparky`, `sparkno`, `guard.sparkno`, `pausetime.p1`, `pausetime.p2`, `guard.pausetime.p1/.p2`, `ground.velocity.x/.y`, `air.velocity.x/.y`, `down.velocity.x/.y`, `guard.velocity`, `attr` and `guardflag` (bit masks as in `GetHitVar`), `id`, `chainid`, `damage`, `guard.damage`, `hittime`, `numhits`, `fall`, `yaccel`. Bare `pausetime` and `*.velocity` mean the first component. Unknown names return 0 |
| `MoveHitVar(<name>)` | Reads the last hit this character landed: `id` and `playerno` of the target, `sparkx`, `sparky` (contact point relative to the attacker), `cornerpush` (push-back applied to the attacker in a corner), `overridden` (the hit was absorbed by a `HitOverride`), `frame` (1 for the tick after the hit) |
| `MoveCountered` | Like `MoveHit`, but only when the target was in an attack (`movetype = A`) when it was hit. Cleared with `MoveHit` |
| `HitByAttr = SCA, NA, SA` | 1 when the last hit taken matches the attribute list (same syntax as `HitDefAttr`) |
| `HitOverridden` | 1 for the tick after a `HitOverride` absorbed a hit |
| `PrevStateType = S`, `PrevMoveType = A` | State type and move type of the state before the current one |
| `PrevAnim` | Animation number before the last animation change |
| `SelfStateNoExist(n)` | 1 when the character's own files define state `n` |
| `InRunState` | 1 in the standard run states (100, 105) |
| `InCustomState` | 1 while running a state owned by another character (`p2getp1state`) |
| `InCustomAnim` | 1 while playing an animation owned by another character (`ChangeAnim2`) |
| `IsAsserted(<flag>)` | 1 when the flag was asserted this tick with `AssertSpecial`. Shares the flag table with `AssertSpecial` (character flags and `TimerFreeze`, `Intro`, `RoundNotOver`) |
| `JugglePoints` | Juggle points left: the attacker's `data.airjuggle` (own when not being hit) minus the points already spent |
| `HitVel X` / `HitVel Y` | Velocity set by the last hit taken |
| `ParentExist` | 1 while the parent of this helper still exists |
| `AirJumpCount` | Air jumps performed since landing |

## Redirections

`parent`, `root`, `helper(id)`, `target(id)`, `partner(n)` (the n-th other non-helper player on the same side, default 0; a helper redirects through its root; NaN when there is none), `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Example: `trigger1 = enemynear, life < 200`.

## Expressions

- Integer vs float like M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Ranges: `stateno = [200, 299]`, `(200, 299]`.
- Operators: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` and `:=` in `var(n) := value`.

## Debugging

- `F2` in training shows hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <folder>` lists states and expressions with errors.
