# AssertSpecial flags

`AssertSpecial` accepts `flag`, `flag2`, `flag3`, `flag4` and `flag5` (a comma list also works in one parameter) and an optional `enabled` (default 1). Flag names are case insensitive. The full Ikemen GO v1.0.0 set is understood: 58 per-character flags and 19 global flags, plus the engine extension `Immovable`.

A flag lasts for the tick in which it is asserted. During hit pause the flags asserted before the pause stay on (Ikemen behavior); `AnimateHitPause` always has to be asserted again. `IsAsserted(<flag>)` reads the same table, so `IsAsserted(NoWalk)` and `IsAsserted(TimerFreeze)` both work.

A flag that is not in the table is ignored. The loader prints one `AssertSpecial flags not recognised and ignored` line per character, and `mugen_check` prints an `UNKNOWN-FLAGS` line for it.

## Global flags (19)

| Flag | Effect |
|---|---|
| `Intro` | Holds the round in the intro (capped at 10 s) |
| `RoundNotOver` | Holds the round-over state (capped at 15 s) |
| `TimerFreeze` | Round timer stops |
| `RoundFreeze` | Round timer and round state clock stop |
| `RoundNotSkip` | The attack-button skip of the first-round face-off is disabled |
| `CameraFreeze` | The camera does not move |
| `GlobalNoKO` | Nobody is KO'd; life stops at 0 and the round goes on |
| `GlobalNoShadow` | No shadows or reflections |
| `NoKOSlow` | No slow motion on KO |
| `NoKOSnd` | No KO sound |
| `NoMusic` | Music is muted |
| `NoBG` / `NoFG` | Back / front stage layers are not drawn |
| `NoBarDisplay` | The whole fight HUD (bars, timer, partner tiles, combo counter) is hidden |
| `NoTimeDisplay` | The timer is hidden |
| `SkipFightDisplay` / `SkipRoundDisplay` / `SkipKODisplay` / `SkipWinDisplay` | The "FIGHT!", "ROUND n", "K.O." or win announcement is not shown |

## Per-character flags (58 + Immovable)

| Flag | Effect |
|---|---|
| `Invisible`, `NoShadow` | Sprite or shadow is not drawn |
| `Unguardable` | The player's attacks cannot be guarded |
| `NoStandGuard`, `NoCrouchGuard`, `NoAirGuard` | The player cannot guard in that state type, and does not enter the guard state |
| `NoJuggleCheck` | Juggle points are not enforced against the player's hits |
| `NoAutoTurn`, `NoWalk`, `NoJump`, `NoAirJump`, `NoCrouch`, `NoStand` | Disable the built-in turn, walk, jump, double jump, crouch and stand-up transitions |
| `NoHardcodedKeys` | Disables every built-in key transition (walk, jump, crouch, guard) |
| `AutoGuard` | Guards as if back were held |
| `NoKO` | This player is not KO'd at 0 life |
| `NoHitDamage`, `NoGuardDamage` | Hits / guarded hits do no life damage to the player |
| `NoGuardKO` | A guarded hit cannot kill the player |
| `NoKOFall` | A KO'ing hit does not force the fall |
| `NoKOVelocity` | A KO'ing hit does not add the extra KO launch velocity |
| `NoFallHitFlag` | The player ignores the `F` hitflag while falling |
| `NoCornerPush` | The player is not pushed back after hitting an opponent in the corner |
| `Immovable` | Engine extension: the opponent's pushbox cannot move the player |
| `AnimateHitPause` | The animation keeps playing during hit pause |
| `AnimFreeze` | The animation stops |
| `NoInput` | Button and direction input is cleared |
| `PostRoundInput` | Commands are read after the round ended |
| `RunFirst`, `RunLast` | The player's states run before / after everyone else's on the next tick |
| `NoDestroySelf` | `DestroySelf` has no effect |
| `NoMakeDust` | The built-in dust effect is not spawned |
| `NoAILevel` | `AILevel` returns 0 for the player |
| `DrawUnder` | The player is drawn behind other fighters |
| `NoLifeBarDisplay`, `NoPowerBarDisplay`, `NoFaceDisplay`, `NoNameDisplay`, `NoWinIconDisplay`, `NoComboDisplay` | The matching HUD part is hidden |

Recorded and readable through `IsAsserted`, but the engine has no matching mechanic yet: `NoAIButtonJam`, `NoAICheat`, `NoBrake`, `NoDizzyPointsDamage`, `NoGuardPointsDamage`, `NoRedLifeDamage`, `NoScore`, `NoFaceP2`, `NoFallCount`, `NoFallDefenceUp`, `NoFastRecoverFromLieDown`, `NoGetUpFromLieDown`, `NoIntroReset`, `NoLifeBarAction`, `NoGuardBarDisplay`, `NoStunBarDisplay`, `NoTurnTarget`, `ProjTypeCollision`, `SizePushOnly`.
