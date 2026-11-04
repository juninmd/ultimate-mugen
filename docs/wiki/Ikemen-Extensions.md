# Ikemen extensions / Extensões do Ikemen

State controllers and triggers from Ikemen GO that characters can use in CNS or ZSS.

Controllers e triggers do Ikemen GO que personagens podem usar em CNS ou ZSS.

| Controller | Parameters | Effect |
|---|---|---|
| `MapSet` | `map`, `value`, `type` (1 = add) | Sets a named value on the player |
| `MapAdd` | `map`, `value` | Adds to a named value |
| `MapReset` | `map` (omit for all keys) | Removes one key, or every key, from the player's map |
| `TeamMapSet` / `TeamMapAdd` / `TeamMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the map shared by every player and helper on the same side |
| `RootMapSet` / `RootMapAdd` / `RootMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the root player's map |
| `ParentMapSet` / `ParentMapAdd` / `ParentMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the parent's map; does nothing without a parent |
| `RootVarSet` / `RootVarAdd` | `var(n)`, `fvar(n)`, `sysvar(n)`, `sysfvar(n)` or `v` / `fv` + `value` | Writes variables on the root player, like `ParentVarSet` does on the parent |
| `RedLifeSet` / `RedLifeAdd` | `value`, `absolute` | Recoverable (red) life, kept between `life` and `lifemax` |
| `DizzyPointsSet` / `DizzyPointsAdd` | `value` | Stun points, `0..data.dizzypoints` |
| `DizzySet` | `value` | Dizzy flag |
| `GuardPointsSet` / `GuardPointsAdd` | `value` | Guard points, `0..data.guardpoints` |
| `GuardBreakSet` | `value` | Guard break flag |
| `Text` | `text`, `params`, `pos`, `velocity`, `scale`, `color`, `align`, `removetime`, `id` | Draws text on the fight screen; `%d`, `%f`, `%s` and `\n` in `text` take values from `params` (`%s` prints the mode name for a bare `TeamMode`, otherwise the integer) |
| `RemoveText` | `id` (omit for all) | Removes the player's texts |
| `TagOut` / `TagIn` | `self`, `stateno`, `ctrl`, `partnerstateno`, `partnerctrl` | Moves the player to or from standby (hidden, cannot be hit or pushed); `partnerstateno` swaps with the first teammate. In [Tag mode](Game-Modes.md#tag-team) they perform a real swap (cooldown ignored): `TagOut` brings the next living teammate in, `TagIn` on a standby fighter brings it in |
| `ModifyProjectile` | `id`, `velocity`, `accel`, `velmul`, `removetime`, `projhits`, `projpriority`, `projscale` | Changes the player's live projectiles |
| `ModifySnd` | `channel`, `pan`, `abspan`, `volumescale` | Changes sounds already playing on a channel |
| `PlayBGM` | `bgm`, `volume`, `loop`, `loopstart`, `loopend`, `freqmul` | Starts a music track (`volume` in percent); an empty `bgm` stops the music. The engine only emits an event; the app resolves the file: `"stage"` is the stage's own track, `"builtin:N"` a built-in track, other names must have been registered by the app |
| `StopBGM` | | Stops the music |
| `ModifyBGM` | `volume`, `loop`, `loopstart`, `loopend`, `freqmul` | `volume` applies to the playing track at once; `loop` is used by the next `PlayBGM`; `loopstart`, `loopend` and `freqmul` are emitted as events but changing them while a track plays is not applied yet |
| `ModifyStageVar` | `camera.boundleft`, `camera.boundright`, `camera.boundhigh`, `camera.boundlow`, `camera.verticalfollow`, `camera.tension`, `camera.startx`, `camera.starty`, `camera.zoomout`, `stageinfo.zoffset`, `playerinfo.p1startx`, `playerinfo.p2startx` | Changes the running stage; the part before the dot is optional. Values are in character units; camera position is clamped to the new bounds at once and `zoffset` is re-applied through the zoom. `startx` / `starty` apply on the next round |
| `ModifyStageBG` | `id` (omit or -1 for all layers), `visible`, `enabled`, `velocity`, `pos`, `offset`, `sinx`, `siny`, `anim` | Changes background layers of the running stage; `pos` is absolute, `offset` is added |
| `ModifyBGCtrl` | `id` (target layer id), `time` (start, end, loop), `looptime`, `value`, `x`, `y` | Edits the stage's `BGCtrl` entries that target `id` (all when omitted); the edits last until the round restarts |
| `ModifyPlayer` | `id` (omit for self), `life`, `lifemax`, `power`, `powermax`, `ctrl`, `ailevel`, `redlife`, `dizzypoints`, `guardpoints` | Whitelisted fields only, always clamped (`ailevel` 0..8, `power` lives on the root) |
| `PrintToConsole` | `text`, `params` | Appends a line to the match's compatibility log (`Match::console`); `%d`, `%f` and `\n` as in `Text`. At most 20 lines per second per match, 200 lines kept, 240 characters per line |
| `RunState` | `value` | Runs the controllers of state `value` (from the state owner) once, in the current tick, without changing `stateno` or `Time`; variables are shared, `persistent` and `ignorehitpause` of the inner controllers are respected, nesting stops after 8 levels |
| `Height` | `value = top[, bottom]` | Sets the body height for the current tick (default `size.height`, bottom 0); used by player push and as the hurtbox of frames that have none |
| `GetHitVarSet` | `hittype`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.damage`, `fall.xvel`, `fall.yvel`, `fall.recover`, `fall.time`, `fall.recovertime`, `fall.kill`, `kill`, `id`, `playerno` | Writes the listed `GetHitVar` fields (`type` is spelled `hittype` because `type` names the controller); `GetHitVar(attr)` is not available yet |
| `ModifyHitDef` | any `HitDef` parameter | Overwrites only the listed fields of the HitDef that is active; nothing happens without one |
| `ModifyReversalDef` | any `ReversalDef` parameter, `reversal.attr` | Same for the pending `ReversalDef` |
| `OverrideClsn` | `clsn` (1 attack, 2 hurt, default 2), `index` (-1 replaces the whole list), `value = x1, y1, x2, y2`, `time` (default 1, -1 = until replaced) | Replaces a collision box in animation units |
| `ResizeClsn` | `clsn`, `index` (-1 = all boxes), `value = dx1, dy1, dx2, dy2`, `time` | Moves each edge of the box by the given deltas |
| `TransformClsn` | `clsn`, `scale = sx[, sy]`, `offset = x, y`, `time` | Scales then shifts every box of that type |

Collision modifiers apply in the order they were added, are replaced when the same kind of change is issued again for the same box type (and `index`), count down once per tick, and are reflected by the F2 box view. `time = -1` keeps a modifier until another one replaces it.

PT-BR: `RunState` executa os controllers de outro state sem trocar de state; `Height` ajusta a altura do corpo no tick; `GetHitVarSet` grava campos do `GetHitVar` (`hittype` no lugar de `type`); `ModifyHitDef` e `ModifyReversalDef` alteram só os campos listados; `OverrideClsn`, `ResizeClsn` e `TransformClsn` (`clsn` = 1 ataque, 2 corpo) mudam as caixas de colisão e aparecem no F2.

| Trigger | Returns |
|---|---|
| `Map(name)` | Named value, 0 when unset (use `Root, Map(name)` or `Parent, Map(name)` to read another scope) |
| `TeamMap(name)` | Value in the team map of the player's side, 0 when unset |
| `RedLife` | Red life |
| `Dizzy`, `DizzyPoints`, `DizzyPointsMax` | Dizzy flag and points |
| `GuardBreak`, `GuardPoints`, `GuardPointsMax` | Guard break flag and points |
| `StandBy` | 1 while tagged out |
| `NumText` | Texts the player has on screen |
| `ClsnOverlap(clsn1, id, clsn2)` | 1 when the player's box type overlaps the box type of the player with that `id` |
| `NumPartner` | Teammates on the same side |
| `IsAsserted(flag)` | 1 while the [AssertSpecial](AssertSpecial.md) flag is on (character or global flag) |
| `TeamMode` | 0 single, 1 simul (Parallel), 2 turns (Sequence), 3 tag (reserved, no tag mode yet). Also accepts `TeamMode = Simul`, `TeamMode = turns`, `TeamMode = 1` (names are case-insensitive) |
| `TeamSize` | Members of the player's team (the whole roster in Sequence mode, the fielded players otherwise) |
| `TeamLeader` | `PlayerNo` of the player's team leader |
| `NumPlayer` | Players in the match (both rosters; 6 for a 3v3 Sequence, 10 for a 5v5 Parallel) |
| `PlayerNo` | 1-based slot of the player: P1, P2, P3... interleaved by side (a helper reports its root's slot); in Sequence mode `member * 2 + side + 1` |
| `MemberNo` | 1-based position of the player in its team (a helper reports its root's position) |
| `PlayerIndexExist(n)` | 1 when the 0-based index `n` exists in the match player list (helpers included) |
| `LastPlayerID` | ID of the most recently created player or helper |
| `IntroState` | 1 before the intro starts, 2 while the intro plays, 0 afterwards |
| `OutroState` | 1 while the round is decided (KO / time over), 2 during the win poses, 0 otherwise |

Map keys are case-insensitive. Each map holds at most 256 keys of at most 64 characters; writes that would exceed either limit are ignored.

Chaves de map não diferenciam maiúsculas. Cada map guarda no máximo 256 chaves de até 64 caracteres; escritas acima desses limites são ignoradas.

`data.dizzypoints` and `data.guardpoints` in the `[Data]` section default to `data.life`.

`data.dizzypoints` e `data.guardpoints` na seção `[Data]` usam `data.life` como padrão.

## Module controllers / Controllers de módulo

Ikemen GO modules and motifs use these controllers. They are always parsed (a character that contains them still loads and runs); the table says what each one does here. A controller that is not supported does nothing and writes one warning line to the match's compatibility log (`Match::console`) the first time it runs. The list lives in `ume_engine::sctrl::module::MODULE_CONTROLLERS` so tools can query it with `module_status(name)` / `rejected_by_design()`.

Controllers de módulo do Ikemen GO são sempre interpretados (o personagem carrega e roda); os não suportados não fazem nada e registram um aviso único no log de compatibilidade.

| Controller | Status | Notes |
|---|---|---|
| `MatchRestart` | supported | Resets wins and the round counter and restarts at the start of the next tick |
| `LoadFile`, `SaveFile` | rejected by design | Characters never get file access; content runs in a sandbox |
| `ShaderSet` | rejected by design | The renderer has a fixed pipeline; user shader code is not loaded |
| `Storyboard`, `RemoveStoryboard`, `Dialogue` | planned | `[SceneDef]` files are parsed but there is no storyboard runtime yet |
| `SaveState`, `LoadState` | planned | Needs the rollback snapshot of the match (see the roadmap) |
| `LifebarAction` | planned | The HUD has no scripting hooks yet |
| `ChangeMovelist` | planned | The movelist is fixed at load time |
