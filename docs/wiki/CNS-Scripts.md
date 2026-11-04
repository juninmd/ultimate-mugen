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

Command extras (Ikemen GO 1.0):

| Feature | Behavior |
|---|---|
| `N` step | Neutral direction: no direction key held (`command = N, x`, `~D, N, x`, `N+x`). A neutral step is a held state, like `/`, so it matches while no direction is pressed |
| `pausebuffer` | `[Command]` (or `command.pausebuffer` in `[Defaults]`) keeps a command that fired during hit pause, hit shake or a pause/super pause active for that many extra ticks, so it can be read when the pause ends. Also accepted as `buffer.pausetime`. Default 0 |
| Custom states | When a player runs a state owned by another character (`TargetState`, `p2getp1state`), `Command` is evaluated with the commands of the state owner against the input of the player in the state |

## Supported state controllers

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `CtrlSet` `DefenceMulSet` `DestroySelf` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `GameMakeAnim` `Gravity` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `ModifyExplod` `MoveHitReset` `NotHitBy` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RemoveExplod` `ScreenBound` `SelfState` `SprPriority` `StateTypeSet` `SuperPause` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `Width`

Also supported: `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.

Explod notes: `ignorehitpause` defaults to 1 (set 0 to freeze with the owner's hit pause). `postype = left/right/front/back` and `space = screen` keep the explod fixed on screen while the camera moves. `ModifyExplod` accepts `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` and `trans`; `facing` is relative to the owner. `ExplodBindTime` only changes the bind time.

## Supported triggers

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Command` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Missing (planned): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

`P3Name` to `P8Name` compare against the player in that slot (slot order of the match, not side); an unused slot never matches. `P2Command` reads the command buffer of the nearest opponent, like `Command` does for the owner.

`TeamMode` returns 0 (single), 1 (simul, Parallel mode), 2 (turns, Sequence mode) or 3 (tag, reserved). `TeamMode = Simul`, `TeamMode = turns` and `TeamMode = 1` are all accepted; see Ikemen-Extensions for the `%s` form in `Text`.

Extra functions: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

`HitCount` sums the `numhits` of every hit the attacker lands. `UniqHitCount` counts each distinct `(target, HitDef id)` pair once, so three hits of one HitDef give `HitCount = 3` and `UniqHitCount = 1`. Both reset together on state changes unless `HitCountPersist = 1`.

`GetHitVar(<name>)` reads the data of the last hit taken (after `HitDef`, guard or `HitOverride`). Beyond the MUGEN 1.1 names (`type`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.*`, `kill`, `id`, `playerno`) these Ikemen names return real values: `recovertime` (ticks left until `fall.recovertime` allows recovery, 0 when not falling), `down.recover` (the HitDef `down.recover`, default 1), `priority`, `facing` (attacker facing, 1 or -1), `power` (power the target received), `isbound` (live, 1 while the target is bound), `attr` (bit mask: S=1 C=2 A=4 L=8, NA=64 NT=128 NP=256 SA=512 ST=1024 SP=2048 HA=4096 HT=8192 HP=16384), `guardflag` (bit mask: H=1 L=2 A=4), `hitdamage`, `guarddamage`, `hitpower`, `guardpower`, `dizzypoints` (HitDef `dizzypoints`, default the hit damage), `guardpoints` (HitDef `guardpoints`, default the guard damage), `redlife` (HitDef `redlife`, hit or guard value), `score` (HitDef `score`, hit or guard value), `teamside` (attacker side, 1 or 2), `projid` (-1 unless a projectile hit), `playerid` (attacker ID, same as `id`), `p1stateno` and `p2stateno` (-1 when the HitDef sets none). Any other name logs one warning on stderr when the expression is loaded and evaluates to 0.

## Redirections

`parent`, `root`, `helper(id)`, `target(id)`, `partner(n)` (the n-th other non-helper player on the same side, default 0; a helper redirects through its root; NaN when there is none), `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Example: `trigger1 = enemynear, life < 200`.

## Expressions

- Integer vs float like M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Ranges: `stateno = [200, 299]`, `(200, 299]`.
- Operators: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` and `:=` in `var(n) := value`.

## Pause and freeze rules

| Entity | Hit pause (attacker) / hit shake (defender) | `Pause` | `SuperPause` |
|---|---|---|---|
| Fighter or helper | Animation, `Time`, physics and timers stop; only controllers with `ignorehitpause = 1` run | Frozen, except the pauser during `movetime` and helpers with `pausemovetime` ticks left | Frozen, except the pauser during `movetime` and helpers with `supermovetime` ticks left |
| Explod | Keeps going with `ignorehitpause = 1` (default); with `0` it stops while its owner is in hit pause or hit shake, position binding included | Frozen unless `pausemovetime` ticks are left | Frozen unless `supermovetime` ticks are left |
| Projectile | Not affected by the owner's hit pause | Frozen unless `pausemovetime` ticks are left | Frozen unless `supermovetime` ticks are left |

- A `pausemovetime`/`supermovetime` budget is spent one tick per frozen frame and only by its own kind of pause.
- `AnimTime`, `AnimElemTime`, `AnimElemNo` and `Time` hold their value for the whole hit pause or shake and resume on the first free tick.
- `HitPauseTime` is the attacker's remaining hit pause; `GetHitVar(hitshaketime)` and `HitShakeOver` describe the defender's shake. Every other `GetHitVar` field is a snapshot taken when the hit lands and does not change during the shake; the recovery clock (`HitOver`) also waits for the shake to end.
- Regression tests: `crates/engine/tests/freeze_rules.rs`.

## Debugging

- `F2` in training shows hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <folder>` lists states and expressions with errors.
