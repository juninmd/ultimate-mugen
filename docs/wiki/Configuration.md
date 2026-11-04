# Configuration and controls

## `ultimate-mugen.cfg`

Lives next to the executable and is written by the **Video** menu. It can be edited by hand.

| Key | Values | Effect |
|---|---|---|
| `fullscreen` | `0` / `1` | Fullscreen |
| `res` | `0` 1280×720, `1` 1920×1080, `2` 2560×1440, `3` 3840×2160, `4` 5120×2880 | Window size |
| `quality` | `0` automatic, `1` 3×, `2` 4.5×, `3` 6× | Vector art density |
| `fps` | `0` / `1` | Show FPS |
| `fx` | `0` to `2` | Post-processing (bloom, vignette, etc.) |
| `impact_lv` | `0` minimum (photosensitivity), `1` reduced, `2` normal, `3` intense | Impact effect strength |

## Command line

| Flag | Effect |
|---|---|
| `--res 2560x1440` | Window resolution |
| `--4k` | 3840×2160 window |
| `--fullscreen` | Fullscreen |
| `--touch` | Force touch controls |
| `--mute` | No audio |
| `--shots <folder>` | Automatic tour saving screenshots |
| `--version` | Print the version |
| `--check-update` / `--no-update-check` | Control the update check |

## Environment variables

| Variable | Effect |
|---|---|
| `UME_MUGEN_DIR` | Alternative M.U.G.E.N content folder |
| `UME_MARKET_HOME` | Folder for [marketplace](Marketplace.md) downloads and state |
| `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` | Optional GitHub token for the marketplace (raises the API rate limit) |
| `UME_NO_UPDATE_CHECK=1` | Skip update checks |
| `UME_NOFX=1` | Disable the animated stage layer (performance testing) |
| `UME_TOUR=mugen` + `UME_CHARS=a,b` + `UME_STAGE=<id>` | With `--shots`, photograph imported fighters |

## Controls

| Action | Keyboard P1 | Keyboard P2 | Gamepad | Touch |
|---|---|---|---|---|
| Move | arrows | `I` `K` `J` `L` | D-pad / stick | virtual stick |
| Punches `x` `y` `z` | `Q` `W` `E` | Num `4` `5` `6` | □ △ R1 | LP HP |
| Kicks `a` `b` `c` | `A` `S` `D` | Num `1` `2` `3` | ✕ ○ R2 | LK HK |
| Quick specials | `1` `2` `3` | Num `7` `8` `9` | L1, L2 | ESPECIAL, SUPER |
| Super | `1`+`2` | Num `7`+`8` | L1+L2 | SUPER |
| Awakening | `T` | Num `0` | R3 / SELECT | — |
| Pause / hitboxes | `Esc` / `F2` | `Esc` / `F2` | START | ❚❚ |
| Fullscreen / FPS | `F11` / `F3` | — | — | — |

The names `x y z a b c` are the same ones used in character commands (`input`).

## Button prompts

The pause move list and the Controls screen show the real key or button of the device you used last: keyboard keys, Xbox (A/B/X/Y, LB/LT/RB/RT) or PlayStation (✕ ○ □ △, L1/L2/R1/R2). On touch screens they show the virtual button names. Xbox pads are detected by name or by vendor ID `045E`, PlayStation pads by `054C`. Rumble works on both: Xbox through gilrs force feedback (and the Gamepad API on the web), DualShock 4 / DualSense through direct HID reports.
