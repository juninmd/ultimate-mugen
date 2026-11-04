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

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AppendToClipboard` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `ClearClipboard` `CtrlSet` `DefenceMulSet` `DestroySelf` `DisplayToClipboard` `DizzyPointsAdd` `DizzyPointsSet` `DizzySet` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `FallEnvShake` `ForceFeedback` `GameMakeAnim` `Gravity` `GuardBreakSet` `GuardPointsAdd` `GuardPointsSet` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitOverride` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `MapAdd` `MapReset` `MapSet` `ModifyExplod` `ModifyProjectile` `ModifySnd` `MoveHitReset` `NotHitBy` `Null` `Offset` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RedLifeAdd` `RedLifeSet` `RemapPal` `RemoveExplod` `RemoveText` `ReversalDef` `RootVarAdd` `RootVarSet` `ScreenBound` `SelfState` `SndPan` `SprPriority` `StateTypeSet` `StopSnd` `SuperPause` `TagIn` `TagOut` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Text` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `VictoryQuote` `Width`

Also supported: `Zoom` `Camera` `GroundLevelOffset` `Depth` `DepthBound` `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.
Limits of the lighter controllers: `Offset` shifts only the drawn sprite, never the collision boxes or the position; `Null` does nothing; `VictoryQuote` stores its `value` on the fighter for the win-quote presentation; `ForceFeedback` becomes a rumble event for the character (`self = 1`) or for its opponent; `StopSnd` and `SndPan` act on the sound channels the character started; `DisplayToClipboard`, `AppendToClipboard` and `ClearClipboard` keep the last 10 lines on the fighter (`%d`, `%i`, `%f`, `%%`, `\n` and `\t` in the text, up to 5 `params`).

Shake and sound notes: `EnvShake` (and the `envshake.*` / `fall.envshake.*` HitDef parameters) take `dir`: `0` (default) shakes the stage on both axes, `1` only horizontally and `2` only vertically; values above 2 count as 2. `PlaySnd` accepts `stopongethit` and `stoponchangestate`. A flagged sound is stopped when the player that started it is hit (or guards) or enters another state; a new `PlaySnd` on the same `channel` replaces the previous flags, and `StopSnd` stops the channel as usual. The flags travel with the sound event, so the engine only reports them and the front end does the stopping.

Explod notes: `ignorehitpause` defaults to 1 (set 0 to freeze with the owner's hit pause). `postype = left/right/front/back` and `space = screen` keep the explod fixed on screen while the camera moves. `ModifyExplod` accepts `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` and `trans`; `facing` is relative to the owner. `ExplodBindTime` only changes the bind time.

Palette effects stack in this order: the stage `BGPalFX`, then `AllPalFX`, then the fighter's own `PalFX`. They apply on the tick they change (no extra frame of latency), and a `Sub` trans skips the palette tint while additive trans keeps it. `hue` (degrees) rotates colours around the grey axis and `sinhue` oscillates that rotation using the `sinadd` period; both work on the three palette controllers.

HitDef life-system parameters: `redlife` raises the defender red-life band, `dizzypoints` and `guardpoints` drain the dizzy and guard meters (zero sets `Dizzy`/`GuardBreak` and enters state 5300/5400 when the character defines it), `score` feeds the attacker and defender scores, `sparkscale` scales the hit spark, `missonoverride` shows a `MISS` text when a `HitOverride` absorbs the hit, `unhittabletime` grants the defender invincibility for N ticks (the countdown pauses during hit pause) and `xaccel` drifts the defender horizontally while in the hit state. `attack.depth` is parsed and ignored (the engine is 2D). `p2clsncheck = 0` lets the hit connect without defender boxes and `p2clsnrequire = n` requires the defender's Clsn2 box `n` (index -1 = any).

Helper notes: a helper freezes during `Pause` and `SuperPause` unless it has `pausemovetime` / `supermovetime` ticks left or was spawned with `pausebool = 1`, which makes it ignore every pause. Helpers with `keyctrl = 1` only receive their root's commands while the round is being fought; once the round is decided they get none. On the victory screen (round state 4) helpers are frozen and hidden.

## Ikemen, stage and player controllers

`PlayBGM` `StopBGM` `ModifyBGM` `ModifyStageVar` `ModifyStageBG` `ModifyBGCtrl` `ModifyPlayer` `PrintToConsole` `RunState` `Height` `GetHitVarSet` `ModifyHitDef` `ModifyReversalDef` `OverrideClsn` `ResizeClsn` `TransformClsn` `TargetAdd` `TargetScoreAdd` `TargetRedLifeAdd` `TargetDizzyPointsAdd` `TargetGuardPointsAdd` `ScoreAdd` `RoundTimeSet` `RoundTimeAdd` `AssertCommand` `AssertInput` `ShiftInput` `RemapSprite` `TransformSprite` `MatchRestart`

Missing (planned or rejected by design, parsed and ignored with one warning): `Storyboard` `RemoveStoryboard` `Dialogue` `SaveState` `LoadState` `LifebarAction` `ChangeMovelist` `LoadFile` `SaveFile` `ShaderSet`

Their parameters and behaviours are listed in [Ikemen extensions](Ikemen-Extensions.md).

## Supported triggers

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimElemVar` `AnimExist` `AnimLength` `AnimPlayerNo` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Clamp` `ClsnOverlap` `ClsnVar` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DisplayName` `Dizzy` `DizzyPoints` `DizzyPointsMax` `DrawGame` `E` `Exp` `ExplodVar` `facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `GuardBreak` `GuardPoints` `GuardPointsMax` `HelperIndexExist` `HelperVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `ID` `IfElse` `IkemenVersion` `Index` `InGuardDist` `InputTime` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `LoseKO` `LoseTime` `MajorVersion` `Map` `MatchNo` `MatchOver` `Max` `Min` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `MugenVersion` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `NumText` `P1Dist` `P1Name` `P2BodyDist` `P2Command` `P2Ctrl` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2Power` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalFXVar` `PalNo` `ParentDist` `Physics` `Pi` `PlayerIDExist` `pos` `Power` `PowerMax` `PrevStateNo` `ProjCancelTime` `ProjClsnOverlap` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `ProjVar` `Random` `RandomRange` `RedLife` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `RunOrder` `ScreenPos` `SelfAnimExist` `SelfCommand` `Sign` `Sin` `SpritePlayerNo` `SpriteVar` `Sqrt` `Standby` `StateNo` `StateTime` `StateType` `SysFVar` `SysVar` `Tan` `TeamMap` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `vel` `Win` `WinKO` `WinPerfect` `WinTime` `AirJumpCount` `Alpha` `Angle` `BotBoundBodyDist` `BotBoundDist` `Const1080p` `HitByAttr` `HitDefVar` `HitOverridden` `HitVel` `InCustomAnim` `InCustomState` `InRunState` `IntroState` `IsAsserted` `JugglePoints` `LastPlayerID` `LayerNo` `LocalCoord` `MemberNo` `MoveCountered` `MoveHitVar` `NumPlayer` `NumStageBG` `OutroState` `ParentExist` `PlayerIndexExist` `PlayerNo` `PrevAnim` `PrevMoveType` `PrevStateType` `Scale` `ScreenHeight` `ScreenWidth` `SelfStateNoExist` `SprPriority` `StageBackEdgeDist` `StageBGVar` `StageConst` `StageFrontEdgeDist` `StageVar` `TeamLeader` `TeamSize` `TopBoundBodyDist` `TopBoundDist` `XAngle` `XShear` `YAngle` `ZoomVar` `Attack` `Defence` `AttackMul` `DefenceMul` `ReceivedDamage` `ReceivedHits` `ComboCount` `Score` `ScoreTotal` `Round` `RoundTime` `TimeRemaining` `TimeElapsed` `TimeTotal` `FightTime` `RoundsWon` `FirstAttack` `DecisiveRound`

Projectile notes: a projectile records its team side and its root player when it is spawned. Turning, or the owner changing side afterwards, does not change whom it can hit. `NumProj` and `NumProjID` count every projectile of the player's root, including the ones its helpers fired, and keep counting them after the helper is destroyed. A projectile whose helper is gone still hits and is credited to the root.

Missing (planned): `PlayerID`.

## Screen edge triggers

`LeftEdge` and `RightEdge` are absolute: they always return the left and right screen edges. `FrontEdge` and `BackEdge` depend on `Facing`: a right-facing player reads `FrontEdge = RightEdge` and `BackEdge = LeftEdge`, a left-facing player reads the opposite. All four are in the player's `localcoord` units and use the same edges as `FrontEdgeDist` and `BackEdgeDist`, so `(FrontEdge - X) * Facing` equals `FrontEdgeDist` (with `X` the player's world position).

## Distances between different localcoords

Every position and distance trigger converts its world-space result into the localcoord of the player that evaluates it (`P2Dist`, `P2BodyDist`, `RootDist`, `ParentDist`, `ScreenPos`, `LeftEdge`/`RightEdge`/`FrontEdge`/`BackEdge` and the `...EdgeDist` family). In a 1280x720 versus 320x240 match the same gap reads 4 times larger for the HD player. `P2BodyDist` subtracts both players' front widths after each width is converted to world units, so it is 0 when the boxes touch no matter which localcoord each character was drawn for. `P2Dist` and `P2BodyDist` only depend on the two player positions: the camera never influences them, including inside a throw's custom states.
## Camera zoom

`Zoom` changes the camera scale (1 = normal, above 1 = closer). The camera is deterministic: it eases one step per tick, with no clock involved.

| Parameter | Default | Meaning |
| --- | --- | --- |
| `scale` | 1 | Requested camera scale |
| `time` | 1 | Ticks the request stays active; re-trigger every tick to hold it |
| `delta` | 0 | Added to the requested scale after every tick of the request (a ramp) |
| `pos` | 0, 0 | Camera focus offset in the player's localcoord |
| `lag` | 0 | 0 jumps to the requested scale in one tick, values towards 1 ease slowly (`1 - lag` of the remaining distance per tick) |
| `zoomindelay` | 0 | Ticks to wait before a zoom-in (`scale > 1`) starts |
| `bound` | 1 | 1 keeps the scale inside the stage limits (`zoomin` as the largest scale, `1 / zoomout` as the smallest, widened by the team zoom-out) and the offset inside the stage bounds; 0 only limits the scale to 0.25..4 and ignores the stage bounds |

When the request ends the camera eases back to its normal scale using the stage `zoominspeed` / `zoomoutspeed`.

`CameraZoom` returns the live camera scale. `ZoomVar(<name>)` exposes the camera zoom state: `scale` (live), `target` (requested scale), `active`, `time` (ticks left), `posx`, `posy`, `lag`, `delta`, `delay`, `bound`, `min` and `max` (the scale limits currently allowed by the stage). All names return 0 (or the live scale for `target`) when no `Zoom` is active.

## Camera and ground level controllers

The engine is 2D, so there is no Z axis: `Pos Z`, `Vel Z` and the other Z components always read 0.

- `Camera` (`pos`, `zoom`, `bound`, `lag`, `time`): moves the camera focus by `pos` (player localcoord) and optionally sets the camera scale with `zoom`. It uses the same machinery as `Zoom` (see above): `bound = 1` keeps the offset inside the stage bounds and the scale inside the stage zoom limits, `lag` eases the zoom, `time` is the number of ticks the request stays active. Without `zoom` the current scale is kept.
- `GroundLevelOffset value = n`: raises the floor under this character by `n` pixels (negative digs it), so it stands on the raised floor with normal landing, ground physics and ground/air hit detection. `Pos Y` is measured from the character's own floor (0 while standing on it), `ScreenPos Y` and the spawn height of projectiles, explods and helpers include the offset. The offset resets at round start.
- `Depth` and `DepthBound` are accepted but ignored (Z is out of scope); the first use prints one warning (`depth is accepted but ignored: the engine has no Z axis`) and the character is not changed.

## Screen, camera and stage triggers

All distances and sizes are in the evaluating player's localcoord. "Zoom-relative" triggers change when the camera zooms (the 5v5 zoom-out, `Zoom`, `Camera`).

| Trigger | Zoom-relative | Meaning |
| --- | --- | --- |
| `ScreenWidth`, `ScreenHeight` | yes | Size of the visible area (`GameWidth` / `GameHeight` keep returning the raw view size in game pixels) |
| `TopBoundDist` | yes | Distance from the player's position up to the highest point the stage lets the camera show (`boundhigh`, raised by `dynamicboundhigh`) |
| `TopBoundBodyDist` | yes | Same, minus the player's `size.height` (head to ceiling) |
| `BotBoundDist`, `BotBoundBodyDist` | yes | Distance from the player's feet down to the lowest point the camera can show (`boundlow` plus the view height); identical because the origin is at the feet |
| `StageFrontEdgeDist`, `StageBackEdgeDist` | yes | Distance to the stage's side limits (`boundleft` / `boundright` extended by half the view) in front of / behind the player, depending on `Facing` |
| `NumStageBG` / `NumStageBG(id)` | no | Number of stage background layers, optionally only those with the given `id` |
| `StageBGVar(id, name)` | no | Live value of the first layer with that `id`: `pos.x`, `pos.y` (start plus scroll and sine offset), `start.x/y`, `delta.x/y`, `velocity.x/y`, `tile.x/y`, `layerno`, `id`, `visible`, `enabled`, `sprite.group`, `sprite.index`; unknown id or name gives 0 |
| `StageConst(name)` | no | Number from the stage's `[Constants]` group (0 when missing or on built-in stages) |
| `Const1080p(n)` | no | `n` scaled from a 1920 pixel wide design to the player's localcoord |
| `LocalCoord X` / `LocalCoord Y` | no | The player's localcoord width / height |
| `Angle`, `Scale X/Y`, `Alpha X/Y`, `SprPriority`, `LayerNo` | no | The player's current draw state (`Alpha` is source / destination, 256 / 0 when opaque). `XAngle`, `YAngle` and `XShear` read 0: the 2D renderer has no tilt or shear |
| `ZoomVar(name)`, `CameraZoom` | yes | See "Camera zoom" |

## Redirections

`parent` `root` `helper` `target` `partner` `enemy` `enemynear` `playerid` `p2` `stateowner`

`helper(id)` and `target(id)` take an optional ID; `partner(n)` is the n-th other non-helper player on the same side (default 0; a helper redirects through its root; NaN when there is none); `enemy(n)` and `enemynear(n)` pick the n-th opponent. Example: `trigger1 = enemynear, life < 200`.

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
