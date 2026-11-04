# Ikemen extensions / Extensões do Ikemen

State controllers and triggers from Ikemen GO that characters can use in CNS or ZSS.

Controllers e triggers do Ikemen GO que personagens podem usar em CNS ou ZSS.

| Controller | Parameters | Effect |
|---|---|---|
| `MapSet` | `map`, `value`, `type` (1 = add) | Sets a named value on the player |
| `MapAdd` | `map`, `value` | Adds to a named value |
| `RedLifeSet` / `RedLifeAdd` | `value`, `absolute` | Recoverable (red) life, kept between `life` and `lifemax` |
| `DizzyPointsSet` / `DizzyPointsAdd` | `value` | Stun points, `0..data.dizzypoints` |
| `DizzySet` | `value` | Dizzy flag |
| `GuardPointsSet` / `GuardPointsAdd` | `value` | Guard points, `0..data.guardpoints` |
| `GuardBreakSet` | `value` | Guard break flag |
| `Text` | `text`, `params`, `pos`, `velocity`, `scale`, `color`, `align`, `removetime`, `id` | Draws text on the fight screen; `%d`, `%f` and `\n` in `text` take values from `params` |
| `RemoveText` | `id` (omit for all) | Removes the player's texts |
| `TagOut` / `TagIn` | `self`, `stateno`, `ctrl`, `partnerstateno`, `partnerctrl` | Moves the player to or from standby (hidden, cannot be hit or pushed); `partnerstateno` swaps with the first teammate |
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

| Trigger | Returns |
|---|---|
| `Map(name)` | Named value, 0 when unset |
| `RedLife` | Red life |
| `Dizzy`, `DizzyPoints`, `DizzyPointsMax` | Dizzy flag and points |
| `GuardBreak`, `GuardPoints`, `GuardPointsMax` | Guard break flag and points |
| `StandBy` | 1 while tagged out |
| `NumText` | Texts the player has on screen |
| `ClsnOverlap(clsn1, id, clsn2)` | 1 when the player's box type overlaps the box type of the player with that `id` |
| `NumPartner` | Teammates on the same side |

`data.dizzypoints` and `data.guardpoints` in the `[Data]` section default to `data.life`.

`data.dizzypoints` e `data.guardpoints` na seção `[Data]` usam `data.life` como padrão.
