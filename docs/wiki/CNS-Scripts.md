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

Also supported: `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`. Ikemen additions (`RunState`, `Height`, `GetHitVarSet`, `ModifyHitDef`, `ModifyReversalDef`, `OverrideClsn`, `ResizeClsn`, `TransformClsn`) are listed in [Ikemen extensions](Ikemen-Extensions).

Explod notes: `ignorehitpause` defaults to 1 (set 0 to freeze with the owner's hit pause). `postype = left/right/front/back` and `space = screen` keep the explod fixed on screen while the camera moves. `ModifyExplod` accepts `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` and `trans`; `facing` is relative to the owner. `ExplodBindTime` only changes the bind time.

## Supported triggers

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Command` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Missing (planned): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

`P3Name` to `P8Name` compare against the player in that slot (slot order of the match, not side); an unused slot never matches. `P2Command` reads the command buffer of the nearest opponent, like `Command` does for the owner.

`TeamMode` returns 0 (single), 1 (simul, Parallel mode), 2 (turns, Sequence mode) or 3 (tag, reserved). `TeamMode = Simul`, `TeamMode = turns` and `TeamMode = 1` are all accepted; see Ikemen-Extensions for the `%s` form in `Text`.

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
## Screen edge triggers

`LeftEdge` and `RightEdge` are absolute: they always return the left and right screen edges. `FrontEdge` and `BackEdge` depend on `Facing`: a right-facing player reads `FrontEdge = RightEdge` and `BackEdge = LeftEdge`, a left-facing player reads the opposite. All four are in the player's `localcoord` units and use the same edges as `FrontEdgeDist` and `BackEdgeDist`, so `(FrontEdge - X) * Facing` equals `FrontEdgeDist` (with `X` the player's world position).

## Distances between different localcoords

Every position and distance trigger converts its world-space result into the localcoord of the player that evaluates it (`P2Dist`, `P2BodyDist`, `RootDist`, `ParentDist`, `ScreenPos`, `LeftEdge`/`RightEdge`/`FrontEdge`/`BackEdge` and the `...EdgeDist` family). In a 1280x720 versus 320x240 match the same gap reads 4 times larger for the HD player. `P2BodyDist` subtracts both players' front widths after each width is converted to world units, so it is 0 when the boxes touch no matter which localcoord each character was drawn for. `P2Dist` and `P2BodyDist` only depend on the two player positions: the camera never influences them, including inside a throw's custom states.

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

## Load diagnostics

Every character load audits the controllers, triggers and parameters it parsed against what the engine actually implements. Nothing is hardcoded in a list: each unknown name is run once against a throw-away match and flagged when the dispatch falls through, so the report follows the engine as controllers and triggers are added.

The audit reports, per source file and state (`.cns`, `.cmd`, `.st`, `.zss`):

| Finding | Meaning |
|---|---|
| unsupported controller | the `type` has no implementation |
| unknown trigger | the trigger name is not evaluated by the engine |
| unsupported redirection / string trigger | a redirection kind (`partner`, ...) or a string comparison (`p3name = ...`) the engine does not resolve |
| unknown parameter | no code path of the controller reads that parameter |
| unknown AssertSpecial flag | the flag is not recognised |

Each load prints one summary on stderr (`n unsupported controllers, m unknown triggers, ...`), deduplicated by name; `TRACE=1` adds the names with the files and states that use them. Set `UME_QUIET_DIAGNOSTICS=1` to silence it.

`mugen_check` prints the summary per character and, with `--strict`, prints the details and exits with a non-zero status when any character has findings:

```bash
cargo run --release -p ume-engine --example mugen_check -- --strict mugen/chars/kfm
```

CI runs it with `--strict` over `crates/engine/tests/data/chars`, so a feature used by the fixture characters that the engine does not implement fails the build. In code, `ume_engine::diagnostics::audit_cns(file, text)` returns the findings for a snippet and `CharDef::diagnostics()` for a loaded character.

## Introspection triggers

Triggers that read back what a character created or what it is drawing. The last argument is a variable name written bare, like Ikemen GO; an axis is a second word (`pos x`). A lookup that finds nothing (no such explod, projectile or box) is undefined (NaN): any comparison with it stays undefined and counts as false, so test for existence with `NumExplod`, `NumProj` or `HelperIndexExist` first when the difference matters. Positions and velocities are in the character's own coordinates, positions follow the `Pos` convention (`x` relative to the camera, `y` relative to the floor).

| Trigger | Variables |
|---|---|
| `ExplodVar(id, index, var)` | `id` `time` `removetime` `bindtime` `bindid` `anim` `animelem` `animelemtime` `animtime` `animplayerno` `spriteplayerno` `facing` `ignorehitpause` `layerno` `pausemovetime` `supermovetime` `sprpriority` `pos x/y/z` `vel x/y/z` `accel x/y/z` `scale x/y`. Reads the `index`-th explod of the caller with that `id` (`id` below 0 matches any) |
| `ProjVar(id, index, var)` | `projid` `time` `projremovetime` `projhits` `projmisstime` `projpriority` `projsprpriority` `projedgebound` `projstagebound` `lowbound` `highbound` `projhitanim` `projremanim` `projcancelanim` `projremove` `anim` `animelem` `facing` `teamside` `pausemovetime` `supermovetime` `pos x/y/z` `vel x/y/z` `accel x/y/z` `velmul x/y` `remvelocity x/y/z` `scale x/y`. Reads the `index`-th live projectile of the caller with that `id` |
| `PalFXVar(var)` | `time` (ticks left) `add.r/g/b` `mul.r/g/b` `color` `invertall`; prefix `bg.` or `all.` for the background and global effects. No effect active reads as the identity (`add` 0, `mul` 256, `color` 256) |
| `ClsnVar(type, index, edge)` | `type` is `clsn1`, `clsn2`, `size` or `dummy`; `edge` is `back`, `front`, `top` or `bottom`, relative to the character and its facing. `size` is the body size box from `size.ground.back/front` and `size.height`; `dummy` has no boxes |
| `ClsnOverlap(type, playerid, type)` | now accepts `size` (both sides) and `dummy` (first side, never overlaps) besides `clsn1` and `clsn2` |
| `ProjClsnOverlap(index, playerid, type)` | the attack box of the caller's `index`-th projectile against a box of that player |
| `HelperVar(var)` | for a helper: `id` `helpertype` `keyctrl` `ownpal` `preserve`; undefined for a non-helper |
| `SpriteVar(var)` | the current sprite: `group` `image` `width` `height` `xoffset` `yoffset` |
| `AnimElemVar(var)` | the current animation element: `group` `image` `time` `hflip` `vflip` `xoffset` `yoffset` `xscale` `yscale` `angle` `alphasource` `alphadest` `numclsn1` `numclsn2` |
| `InputTime(key)` | ticks the key has been held: `U` `D` `L` `R`, relative `F` `B`, `N` (nothing pressed) and the buttons `a` `b` `c` `x` `y` `z` `s` (case matters: `B` is back, `b` is the button) |
| `AnimLength` `AnimPlayerNo` `SpritePlayerNo` | total ticks of the animation (-1 when it never ends); player number that owns the animation and the sprites |
| `Index` `RunOrder` `HelperIndexExist(n)` | position of the character among the root and its helpers (root is 0); 1-based update order in the match; whether the root has an `n`-th (0-based) helper |
| `IkemenVersion(major)` `IkemenVersion(minor)` | version of the Ikemen GO behaviour followed: 1 and 0 |
| `DisplayName = "x"` `SelfCommand = "x"` | string comparisons against the display name and the character's own command buffer |

Not implemented yet, and reported by the load diagnostics: `BGMVar`, `SoundVar` (the engine core holds no audio state), `AnimLoopCount`, `AnimPrefix`, `HelperName`, the `angle`, `friction`, `xshear`, `drawpal.*` and `shader` variables of `ExplodVar`, and the `attr`, `guardflag`, `hitflag`, `shadow` variables of `ProjVar`. An unknown variable name is reported once per character load as `explodvar(name)` and evaluates to 0.
